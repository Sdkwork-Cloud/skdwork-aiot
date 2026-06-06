use std::collections::BTreeMap;
use std::sync::Arc;

use sdkwork_aiot_contract::AiotRequestContext;
use sdkwork_aiot_protocol::{InboundFrame, MessageCodec};
use sdkwork_aiot_runtime::{
    standard_aiot_runtime, AiotGatewayListenerBundle, AiotHealthCheck, AiotProtocolRoute,
    AiotRuntime, RuntimeMode,
};
use sdkwork_aiot_runtime::{AiotGatewayPipelineResult, RuntimeProtocolError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpStatus {
    Ok,
    Created,
    Accepted,
    NoContent,
    SwitchingProtocols,
    BadRequest,
    Conflict,
    Unauthorized,
    Forbidden,
    NotFound,
    InternalServerError,
}

impl HttpStatus {
    pub fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::Created => 201,
            Self::Accepted => 202,
            Self::NoContent => 204,
            Self::SwitchingProtocols => 101,
            Self::BadRequest => 400,
            Self::Conflict => 409,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::InternalServerError => 500,
        }
    }

    pub fn reason(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Created => "Created",
            Self::Accepted => "Accepted",
            Self::NoContent => "No Content",
            Self::SwitchingProtocols => "Switching Protocols",
            Self::BadRequest => "Bad Request",
            Self::Conflict => "Conflict",
            Self::Unauthorized => "Unauthorized",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "Not Found",
            Self::InternalServerError => "Internal Server Error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: String,
    pub raw_path: String,
    pub path: String,
    pub body: Vec<u8>,
    headers: BTreeMap<String, String>,
    query_params: BTreeMap<String, String>,
}

impl HttpRequest {
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            method: method.into(),
            raw_path: path.clone(),
            path: path.into(),
            body: Vec::new(),
            headers: BTreeMap::new(),
            query_params: BTreeMap::new(),
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers
            .insert(name.into().to_ascii_lowercase(), value.into());
        self
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }

    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query_params.get(name).map(String::as_str)
    }

    pub fn with_query_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.insert(name.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: HttpStatus,
    headers: BTreeMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn new(status: HttpStatus) -> Self {
        Self {
            status,
            headers: BTreeMap::new(),
            body: String::new(),
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers
            .insert(name.into().to_ascii_lowercase(), value.into());
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }

    pub fn headers(&self) -> &BTreeMap<String, String> {
        &self.headers
    }
}

pub fn build_health_response(component_name: &str, ready: bool) -> HttpResponse {
    let body = format!(r#"{{"component":"{component_name}","ready":{ready}}}"#);

    HttpResponse::new(HttpStatus::Ok)
        .with_header("content-type", "application/json")
        .with_body(body)
}

pub fn handle_http_request_bytes(
    server: &TransportServer,
    bytes: &[u8],
) -> Result<String, TransportError> {
    let request = parse_http_request_bytes(bytes)?;
    let response = if matches!(request.path.as_str(), "/healthz" | "/readyz") {
        build_health_response(&server.health.component_name, server.health.ready)
    } else if server
        .listeners
        .websocket_routes
        .contains(&request.path.as_str())
    {
        build_websocket_handshake_response(&request)?
    } else if let Some(handler) = server.http_compatibility_route(&request.path) {
        handler.as_ref()(&request)
    } else {
        HttpResponse::new(HttpStatus::BadRequest)
        .with_header("content-type", "application/problem+json")
        .with_body(
            r#"{"type":"about:blank","title":"Bad Request","status":400,"code":"transport.route.unsupported"}"#,
        )
    };

    Ok(format_http_response(&response))
}

pub fn parse_http_request_bytes(bytes: &[u8]) -> Result<HttpRequest, TransportError> {
    parse_http_request_prefix(bytes).map(|(request, _)| request)
}

pub fn parse_http_request_prefix(bytes: &[u8]) -> Result<(HttpRequest, usize), TransportError> {
    let header_len = http_header_len(bytes)
        .ok_or_else(|| TransportError::new("transport.http.incomplete_headers"))?;
    let raw = std::str::from_utf8(&bytes[..header_len])
        .map_err(|_| TransportError::new("transport.http.invalid_utf8"))?;
    let mut lines = raw.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| TransportError::new("transport.http.empty"))?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| TransportError::new("transport.http.missing_method"))?;
    let path = parts
        .next()
        .ok_or_else(|| TransportError::new("transport.http.missing_path"))?;
    let mut request = http_request_from_target(method, path);

    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            request = request.with_header(name.trim(), value.trim());
        }
    }

    let mut request = request;
    request.body.extend_from_slice(&bytes[header_len..]);

    Ok((request, header_len))
}

fn http_request_from_target(method: &str, target: &str) -> HttpRequest {
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let mut request = HttpRequest::new(method, path);
    request.raw_path = target.to_string();

    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        request = request.with_query_param(percent_decode(name), percent_decode(value));
    }

    request
}

fn http_header_len(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
}

fn format_http_response(response: &HttpResponse) -> String {
    let mut out = format!(
        "HTTP/1.1 {} {}\r\n",
        response.status.code(),
        response.status.reason()
    );
    let mut has_content_length = false;
    for (name, value) in &response.headers {
        if name == "content-length" {
            has_content_length = true;
        }
        out.push_str(name);
        out.push_str(": ");
        out.push_str(value);
        out.push_str("\r\n");
    }
    if !has_content_length {
        out.push_str("content-length: ");
        out.push_str(response.body.len().to_string().as_str());
        out.push_str("\r\n");
    }
    out.push_str("\r\n");
    out.push_str(&response.body);
    out
}

pub fn build_websocket_handshake_response(
    request: &HttpRequest,
) -> Result<HttpResponse, TransportError> {
    if request.method != "GET" {
        return Err(TransportError::new("transport.websocket.method"));
    }

    let upgrade = request.header("upgrade").unwrap_or_default();
    let connection = request.header("connection").unwrap_or_default();
    let key = request
        .header("sec-websocket-key")
        .ok_or_else(|| TransportError::new("transport.websocket.missing_key"))?;
    let version = request.header("sec-websocket-version").unwrap_or_default();

    if !upgrade.eq_ignore_ascii_case("websocket")
        || !connection.to_ascii_lowercase().contains("upgrade")
        || version != "13"
    {
        return Err(TransportError::new("transport.websocket.invalid_upgrade"));
    }

    Ok(HttpResponse::new(HttpStatus::SwitchingProtocols)
        .with_header("upgrade", "websocket")
        .with_header("connection", "Upgrade")
        .with_header("sec-websocket-accept", websocket_accept_key(key)))
}

fn websocket_accept_key(client_key: &str) -> String {
    let mut data = Vec::with_capacity(client_key.len() + 36);
    data.extend_from_slice(client_key.as_bytes());
    data.extend_from_slice(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    base64_encode(&sha1_digest(&data))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketOpcode {
    Text,
    Binary,
    Close,
    Ping,
    Pong,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketFrame {
    pub opcode: WebSocketOpcode,
    pub payload: Vec<u8>,
}

impl WebSocketFrame {
    pub fn text(text: impl AsRef<str>) -> Self {
        Self {
            opcode: WebSocketOpcode::Text,
            payload: text.as_ref().as_bytes().to_vec(),
        }
    }
}

pub fn websocket_frame_to_inbound_frame(frame: WebSocketFrame) -> InboundFrame {
    InboundFrame {
        binary: frame.opcode == WebSocketOpcode::Binary,
        payload: frame.payload,
    }
}

pub fn encode_websocket_frame(frame: &WebSocketFrame) -> Vec<u8> {
    let opcode = match frame.opcode {
        WebSocketOpcode::Text => 0x1,
        WebSocketOpcode::Binary => 0x2,
        WebSocketOpcode::Close => 0x8,
        WebSocketOpcode::Ping => 0x9,
        WebSocketOpcode::Pong => 0xa,
    };
    let mut out = Vec::with_capacity(2 + frame.payload.len() + 8);
    out.push(0x80 | opcode);

    let payload_len = frame.payload.len();
    if payload_len <= 125 {
        out.push(payload_len as u8);
    } else if payload_len <= u16::MAX as usize {
        out.push(126);
        out.extend_from_slice(&(payload_len as u16).to_be_bytes());
    } else {
        out.push(127);
        out.extend_from_slice(&(payload_len as u64).to_be_bytes());
    }

    out.extend_from_slice(&frame.payload);
    out
}

pub fn handle_websocket_message_bytes<C>(
    server: &TransportServer,
    path: &str,
    codec: &C,
    bytes: &[u8],
) -> Result<AiotGatewayPipelineResult, TransportError>
where
    C: MessageCodec,
{
    let frame = decode_websocket_frame(bytes)?;
    let inbound = websocket_frame_to_inbound_frame(frame);

    server
        .runtime
        .handle_inbound_frame_with_codec(path, codec, inbound)
        .map_err(TransportError::from_runtime_protocol)
}

pub fn handle_websocket_message_bytes_with_context<C>(
    server: &TransportServer,
    path: &str,
    ctx: &AiotRequestContext,
    codec: &C,
    bytes: &[u8],
) -> Result<AiotGatewayPipelineResult, TransportError>
where
    C: MessageCodec,
{
    let frame = decode_websocket_frame(bytes)?;
    let inbound = websocket_frame_to_inbound_frame(frame);

    server
        .runtime
        .handle_inbound_frame_with_context(path, ctx, codec, inbound)
        .map_err(TransportError::from_runtime_protocol)
}

pub fn decode_websocket_frame(bytes: &[u8]) -> Result<WebSocketFrame, TransportError> {
    let (frame, used) = decode_websocket_frame_prefix(bytes)?;
    if used != bytes.len() {
        return Err(TransportError::new("transport.websocket.trailing_bytes"));
    }
    Ok(frame)
}

pub fn decode_websocket_frame_prefix(
    bytes: &[u8],
) -> Result<(WebSocketFrame, usize), TransportError> {
    if bytes.len() < 2 {
        return Err(TransportError::new("transport.websocket.incomplete_frame"));
    }

    let opcode = match bytes[0] & 0x0f {
        0x1 => WebSocketOpcode::Text,
        0x2 => WebSocketOpcode::Binary,
        0x8 => WebSocketOpcode::Close,
        0x9 => WebSocketOpcode::Ping,
        0xa => WebSocketOpcode::Pong,
        _ => {
            return Err(TransportError::new(
                "transport.websocket.unsupported_opcode",
            ))
        }
    };

    let masked = bytes[1] & 0x80 != 0;
    let mut offset = 2usize;
    let mut length = (bytes[1] & 0x7f) as usize;

    if length == 126 {
        if bytes.len() < offset + 2 {
            return Err(TransportError::new("transport.websocket.incomplete_frame"));
        }
        length = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
        offset += 2;
    } else if length == 127 {
        if bytes.len() < offset + 8 {
            return Err(TransportError::new("transport.websocket.incomplete_frame"));
        }
        let extended = u64::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        length = usize::try_from(extended)
            .map_err(|_| TransportError::new("transport.websocket.frame_too_large"))?;
        offset += 8;
    }

    let mask = if masked {
        if bytes.len() < offset + 4 {
            return Err(TransportError::new("transport.websocket.incomplete_frame"));
        }
        let mask = [
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ];
        offset += 4;
        Some(mask)
    } else {
        None
    };

    if bytes.len() < offset + length {
        return Err(TransportError::new("transport.websocket.incomplete_frame"));
    }

    let mut payload = bytes[offset..offset + length].to_vec();
    if let Some(mask) = mask {
        for (index, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[index % 4];
        }
    }

    Ok((WebSocketFrame { opcode, payload }, offset + length))
}

#[derive(Clone)]
pub struct TransportServer {
    pub runtime: AiotRuntime,
    pub listeners: AiotGatewayListenerBundle,
    pub health: AiotHealthCheck,
    http_compatibility_routes: BTreeMap<String, CompatibilityHttpRouteHandler>,
}

pub type CompatibilityHttpRouteHandler =
    Arc<dyn Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static>;

impl TransportServer {
    pub fn standard_standalone() -> Result<Self, TransportError> {
        let runtime = standard_aiot_runtime(RuntimeMode::Standalone)
            .map_err(|_| TransportError::new("transport.runtime.build"))?;

        Ok(Self {
            runtime,
            listeners: AiotGatewayListenerBundle::standard(),
            health: AiotHealthCheck::ready("sdkwork-aiot-transport"),
            http_compatibility_routes: BTreeMap::new(),
        })
    }

    pub fn with_http_compatibility_route(
        mut self,
        path: impl Into<String>,
        handler: impl Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    ) -> Self {
        self.http_compatibility_routes
            .insert(path.into(), Arc::new(handler));
        self
    }

    pub fn http_compatibility_route(&self, path: &str) -> Option<CompatibilityHttpRouteHandler> {
        self.http_compatibility_routes.get(path).cloned()
    }

    pub fn protocol_route_for_path(&self, path: &str) -> Option<&AiotProtocolRoute> {
        self.runtime.protocol_route_for_path(path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportError {
    pub code: String,
}

impl TransportError {
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }

    pub fn from_runtime_protocol(error: RuntimeProtocolError) -> Self {
        Self { code: error.code }
    }
}

fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);

    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | b2 as u32;

        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }

    out
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hi = hex_value(bytes[index + 1]);
                let lo = hex_value(bytes[index + 2]);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    out.push((hi << 4) | lo);
                    index += 3;
                } else {
                    out.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8_lossy(&out).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn sha1_digest(input: &[u8]) -> [u8; 20] {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xefcdab89;
    let mut h2: u32 = 0x98badcfe;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xc3d2e1f0;

    let bit_len = (input.len() as u64) * 8;
    let mut msg = input.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for (i, word) in w.iter_mut().take(16).enumerate() {
            let offset = i * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for (i, word) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5a827999),
                20..=39 => (b ^ c ^ d, 0x6ed9eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdc),
                _ => (b ^ c ^ d, 0xca62c1d6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}
