use std::collections::VecDeque;
use std::io::{self, Stdout};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event as CEvent, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Terminal;
use serde_json::json;
use tungstenite::client::IntoClientRequest;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

const DEFAULT_GATEWAY_HTTP: &str = "http://127.0.0.1:18080";
const DEFAULT_PROTOCOL_VERSION: &str = "3";
const DEFAULT_DEVICE_ID: &str = "sim-device-001";
const DEFAULT_CLIENT_ID: &str = "sim-client-001";
const DEFAULT_TOKEN: &str = "simulator-token";
const LOG_CAPACITY: usize = 200;

#[derive(Debug)]
enum UiCommand {
    ToggleConnect,
    SendHello,
    SendListen,
    SendAbort,
    SendMcpInitialize,
    SendMcpToolsList,
    SendMcpToolsCall,
    Quit,
}

#[derive(Debug)]
enum UiEvent {
    Connected,
    Disconnected(String),
    Outbound(String),
    Inbound(String),
    Error(String),
}

#[derive(Debug, Clone)]
struct SimulatorConfig {
    gateway_http_base: String,
    protocol_version: String,
    device_id: String,
    client_id: String,
    token: String,
}

impl SimulatorConfig {
    fn from_env() -> Self {
        Self {
            gateway_http_base: std::env::var("SDKWORK_AIOT_XIAOZHI_SIMULATOR_GATEWAY_HTTP")
                .unwrap_or_else(|_| DEFAULT_GATEWAY_HTTP.to_string()),
            protocol_version: std::env::var("SDKWORK_AIOT_XIAOZHI_SIMULATOR_PROTOCOL_VERSION")
                .unwrap_or_else(|_| DEFAULT_PROTOCOL_VERSION.to_string()),
            device_id: std::env::var("SDKWORK_AIOT_XIAOZHI_SIMULATOR_DEVICE_ID")
                .unwrap_or_else(|_| DEFAULT_DEVICE_ID.to_string()),
            client_id: std::env::var("SDKWORK_AIOT_XIAOZHI_SIMULATOR_CLIENT_ID")
                .unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string()),
            token: std::env::var("SDKWORK_AIOT_XIAOZHI_SIMULATOR_TOKEN")
                .unwrap_or_else(|_| DEFAULT_TOKEN.to_string()),
        }
    }

    fn websocket_url(&self) -> String {
        let mut base = self
            .gateway_http_base
            .trim()
            .trim_end_matches('/')
            .to_string();
        if base.starts_with("https://") {
            base = base.replacen("https://", "wss://", 1);
        } else if base.starts_with("http://") {
            base = base.replacen("http://", "ws://", 1);
        } else if !base.starts_with("ws://") && !base.starts_with("wss://") {
            base = format!("ws://{base}");
        }

        format!(
            "{base}/iot/xiaozhi/ws?protocol_version={}&device_id={}&client_id={}&token={}",
            encode_query_component(&self.protocol_version),
            encode_query_component(&self.device_id),
            encode_query_component(&self.client_id),
            encode_query_component(&self.token)
        )
    }
}

struct AppState {
    config: SimulatorConfig,
    connected: bool,
    logs: VecDeque<String>,
}

impl AppState {
    fn new(config: SimulatorConfig) -> Self {
        let mut logs = VecDeque::new();
        logs.push_back(format!("gateway={}", config.gateway_http_base));
        logs.push_back(format!("websocket={}", config.websocket_url()));
        Self {
            config,
            connected: false,
            logs,
        }
    }

    fn push_log(&mut self, text: impl Into<String>) {
        if self.logs.len() >= LOG_CAPACITY {
            self.logs.pop_front();
        }
        self.logs.push_back(text.into());
    }
}

fn main() {
    let config = SimulatorConfig::from_env();
    let (command_tx, command_rx) = mpsc::channel::<UiCommand>();
    let (event_tx, event_rx) = mpsc::channel::<UiEvent>();

    spawn_transport_worker(config.clone(), command_rx, event_tx);

    if let Err(error) = run_ui_loop(config, command_tx, event_rx) {
        eprintln!("sdkwork-aiot-xiaozhi-simulator-ui error={error}");
    }
}

fn spawn_transport_worker(
    config: SimulatorConfig,
    command_rx: Receiver<UiCommand>,
    event_tx: Sender<UiEvent>,
) {
    thread::spawn(move || {
        let mut socket: Option<WebSocket<MaybeTlsStream<TcpStream>>> = None;
        loop {
            while let Ok(command) = command_rx.try_recv() {
                match command {
                    UiCommand::ToggleConnect => {
                        if socket.is_some() {
                            let _ = close_socket(&mut socket);
                            let _ =
                                event_tx.send(UiEvent::Disconnected("closed by user".to_string()));
                            continue;
                        }
                        match open_socket(&config) {
                            Ok(ws) => {
                                socket = Some(ws);
                                let _ = event_tx.send(UiEvent::Connected);
                            }
                            Err(error) => {
                                let _ = event_tx
                                    .send(UiEvent::Error(format!("connect failed: {error}")));
                            }
                        }
                    }
                    UiCommand::SendHello => {
                        if let Err(error) =
                            send_json(&mut socket, &event_tx, hello_message(&config))
                        {
                            let _ = event_tx.send(UiEvent::Error(error));
                        }
                    }
                    UiCommand::SendListen => {
                        if let Err(error) = send_json(
                            &mut socket,
                            &event_tx,
                            json!({
                                "session_id": session_id(&config),
                                "type": "listen",
                                "state": "start"
                            }),
                        ) {
                            let _ = event_tx.send(UiEvent::Error(error));
                        }
                    }
                    UiCommand::SendAbort => {
                        if let Err(error) = send_json(
                            &mut socket,
                            &event_tx,
                            json!({
                                "session_id": session_id(&config),
                                "type": "abort"
                            }),
                        ) {
                            let _ = event_tx.send(UiEvent::Error(error));
                        }
                    }
                    UiCommand::SendMcpInitialize => {
                        if let Err(error) = send_json(
                            &mut socket,
                            &event_tx,
                            json!({
                                "session_id": session_id(&config),
                                "type": "mcp",
                                "payload": {
                                    "jsonrpc":"2.0",
                                    "id": 1,
                                    "method":"initialize",
                                    "params":{
                                        "capabilities":{"vision":{}}
                                    }
                                }
                            }),
                        ) {
                            let _ = event_tx.send(UiEvent::Error(error));
                        }
                    }
                    UiCommand::SendMcpToolsList => {
                        if let Err(error) = send_json(
                            &mut socket,
                            &event_tx,
                            json!({
                                "session_id": session_id(&config),
                                "type": "mcp",
                                "payload": {
                                    "jsonrpc":"2.0",
                                    "id": 2,
                                    "method":"tools/list",
                                    "params":{"cursor":"","withUserTools":false}
                                }
                            }),
                        ) {
                            let _ = event_tx.send(UiEvent::Error(error));
                        }
                    }
                    UiCommand::SendMcpToolsCall => {
                        if let Err(error) = send_json(
                            &mut socket,
                            &event_tx,
                            json!({
                                "session_id": session_id(&config),
                                "type": "mcp",
                                "payload": {
                                    "jsonrpc":"2.0",
                                    "id": 3,
                                    "method":"tools/call",
                                    "params":{
                                        "name":"self.audio_speaker.set_volume",
                                        "arguments":{"volume": 60}
                                    }
                                }
                            }),
                        ) {
                            let _ = event_tx.send(UiEvent::Error(error));
                        }
                    }
                    UiCommand::Quit => {
                        let _ = close_socket(&mut socket);
                        return;
                    }
                }
            }

            if let Some(ws) = socket.as_mut() {
                match ws.read() {
                    Ok(message) => match message {
                        Message::Text(text) => {
                            let _ = event_tx.send(UiEvent::Inbound(text.to_string()));
                        }
                        Message::Binary(payload) => {
                            let _ = event_tx.send(UiEvent::Inbound(format!(
                                "<binary {} bytes>",
                                payload.len()
                            )));
                        }
                        Message::Close(frame) => {
                            let reason = frame
                                .map(|value| format!("server close: {}", value.reason))
                                .unwrap_or_else(|| "server close".to_string());
                            let _ = event_tx.send(UiEvent::Disconnected(reason));
                            socket = None;
                        }
                        Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {}
                    },
                    Err(tungstenite::Error::Io(error))
                        if error.kind() == io::ErrorKind::WouldBlock
                            || error.kind() == io::ErrorKind::TimedOut => {}
                    Err(error) => {
                        let _ =
                            event_tx.send(UiEvent::Disconnected(format!("socket error: {error}")));
                        socket = None;
                    }
                }
            } else {
                thread::sleep(Duration::from_millis(40));
            }
        }
    });
}

fn run_ui_loop(
    config: SimulatorConfig,
    command_tx: Sender<UiCommand>,
    event_rx: Receiver<UiEvent>,
) -> io::Result<()> {
    let mut terminal = init_terminal()?;
    let mut app = AppState::new(config);
    let tick_rate = Duration::from_millis(80);

    let result = loop {
        while let Ok(event) = event_rx.try_recv() {
            match event {
                UiEvent::Connected => {
                    app.connected = true;
                    app.push_log("connected");
                }
                UiEvent::Disconnected(reason) => {
                    app.connected = false;
                    app.push_log(format!("disconnected: {reason}"));
                }
                UiEvent::Outbound(text) => app.push_log(format!(">> {text}")),
                UiEvent::Inbound(text) => app.push_log(format!("<< {text}")),
                UiEvent::Error(error) => app.push_log(format!("!! {error}")),
            }
        }

        terminal.draw(|frame| draw_ui(frame, &app))?;

        if event::poll(tick_rate)? {
            if let CEvent::Key(key) = event::read()? {
                let command = if key.modifiers.contains(KeyModifiers::CONTROL)
                    && matches!(key.code, KeyCode::Char('c'))
                {
                    Some(UiCommand::Quit)
                } else {
                    match key.code {
                        KeyCode::Char('q') => Some(UiCommand::Quit),
                        KeyCode::Char('c') => Some(UiCommand::ToggleConnect),
                        KeyCode::Char('1') => Some(UiCommand::SendHello),
                        KeyCode::Char('2') => Some(UiCommand::SendListen),
                        KeyCode::Char('3') => Some(UiCommand::SendAbort),
                        KeyCode::Char('4') => Some(UiCommand::SendMcpInitialize),
                        KeyCode::Char('5') => Some(UiCommand::SendMcpToolsList),
                        KeyCode::Char('6') => Some(UiCommand::SendMcpToolsCall),
                        KeyCode::Esc => Some(UiCommand::Quit),
                        _ => None,
                    }
                };

                if let Some(command) = command {
                    let quitting = matches!(command, UiCommand::Quit);
                    let _ = command_tx.send(command);
                    if quitting {
                        break Ok(());
                    }
                }
            }
        }
    };

    restore_terminal(&mut terminal)?;
    result
}

fn draw_ui(frame: &mut ratatui::Frame<'_>, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(10)])
        .split(frame.area());

    let status = if app.connected {
        "CONNECTED"
    } else {
        "DISCONNECTED"
    };

    let header_lines = vec![
        Line::from(vec![
            Span::styled(
                "SDKWork Xiaozhi Cross-Platform Simulator",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::raw(format!("status={status}")),
        ]),
        Line::from(Span::raw(format!(
            "gateway={}  device={}  client={}",
            app.config.gateway_http_base, app.config.device_id, app.config.client_id
        ))),
        Line::from(Span::raw(
            "keys: [c] connect/disconnect  [1] hello  [2] listen  [3] abort  [4] mcp.initialize  [5] mcp.tools/list  [6] mcp.tools/call  [q] quit",
        )),
    ];

    let header = Paragraph::new(header_lines)
        .block(Block::default().title("Control").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(header, chunks[0]);

    let log_lines = app
        .logs
        .iter()
        .rev()
        .take((chunks[1].height as usize).saturating_sub(2))
        .rev()
        .map(|entry| Line::from(Span::raw(entry.as_str())))
        .collect::<Vec<_>>();
    let logs = Paragraph::new(log_lines)
        .block(Block::default().title("Session Log").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(logs, chunks[1]);
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn open_socket(
    config: &SimulatorConfig,
) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, tungstenite::Error> {
    let mut request = config.websocket_url().into_client_request()?;
    request
        .headers_mut()
        .insert("Host", host_from_base(&config.gateway_http_base).parse()?);
    request
        .headers_mut()
        .insert("Protocol-Version", config.protocol_version.parse()?);
    request
        .headers_mut()
        .insert("Device-Id", config.device_id.parse()?);
    request
        .headers_mut()
        .insert("Client-Id", config.client_id.parse()?);
    request
        .headers_mut()
        .insert("Authorization", config.token.parse()?);

    let (socket, _) = connect(request)?;
    Ok(socket)
}

fn close_socket(
    socket: &mut Option<WebSocket<MaybeTlsStream<TcpStream>>>,
) -> Result<(), tungstenite::Error> {
    if let Some(ws) = socket.as_mut() {
        ws.close(None)?;
    }
    *socket = None;
    Ok(())
}

fn send_json(
    socket: &mut Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    event_tx: &Sender<UiEvent>,
    payload: serde_json::Value,
) -> Result<(), String> {
    let text = payload.to_string();
    let ws = socket
        .as_mut()
        .ok_or_else(|| "not connected, press 'c' first".to_string())?;
    ws.send(Message::Text(text.clone().into()))
        .map_err(|error| format!("write failed: {error}"))?;
    let _ = event_tx.send(UiEvent::Outbound(text));
    Ok(())
}

fn hello_message(config: &SimulatorConfig) -> serde_json::Value {
    json!({
        "type": "hello",
        "transport":"websocket",
        "version": config.protocol_version.parse::<u32>().unwrap_or(3),
        "features":{"mcp":true},
        "audio_params": {
            "format":"opus",
            "sample_rate": 16000,
            "channels": 1,
            "frame_duration": 60
        }
    })
}

fn session_id(config: &SimulatorConfig) -> String {
    format!("{}-{}", config.device_id, config.client_id)
}

fn encode_query_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        let is_unreserved =
            matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~');
        if is_unreserved {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{byte:02X}"));
        }
    }
    encoded
}

fn host_from_base(base: &str) -> String {
    let trimmed = base.trim();
    let uri_text = if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("ws://")
        || trimmed.starts_with("wss://")
    {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    };

    if let Ok(uri) = uri_text.parse::<http::Uri>() {
        if let Some(authority) = uri.authority() {
            return authority.to_string();
        }
    }

    let no_scheme = trimmed
        .strip_prefix("http://")
        .or_else(|| trimmed.strip_prefix("https://"))
        .or_else(|| trimmed.strip_prefix("ws://"))
        .or_else(|| trimmed.strip_prefix("wss://"))
        .unwrap_or(trimmed);
    no_scheme
        .split(&['/', '?', '#'][..])
        .next()
        .unwrap_or(no_scheme)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config(gateway_http_base: &str, protocol_version: &str) -> SimulatorConfig {
        SimulatorConfig {
            gateway_http_base: gateway_http_base.to_string(),
            protocol_version: protocol_version.to_string(),
            device_id: "device-001".to_string(),
            client_id: "client-001".to_string(),
            token: "token-001".to_string(),
        }
    }

    #[test]
    fn websocket_url_normalizes_scheme_and_trims_trailing_slash() {
        let http = sample_config("http://127.0.0.1:18080/", "3");
        assert_eq!(
            http.websocket_url(),
            "ws://127.0.0.1:18080/iot/xiaozhi/ws?protocol_version=3&device_id=device-001&client_id=client-001&token=token-001"
        );

        let https = sample_config("https://gateway.example.com/", "3");
        assert_eq!(
            https.websocket_url(),
            "wss://gateway.example.com/iot/xiaozhi/ws?protocol_version=3&device_id=device-001&client_id=client-001&token=token-001"
        );

        let no_scheme = sample_config("gateway.local:19090", "3");
        assert_eq!(
            no_scheme.websocket_url(),
            "ws://gateway.local:19090/iot/xiaozhi/ws?protocol_version=3&device_id=device-001&client_id=client-001&token=token-001"
        );
    }

    #[test]
    fn websocket_url_encodes_query_components() {
        let encoded = SimulatorConfig {
            gateway_http_base: "http://127.0.0.1:18080".to_string(),
            protocol_version: "3 beta".to_string(),
            device_id: "device id".to_string(),
            client_id: "client/001".to_string(),
            token: "token+abc?".to_string(),
        };
        assert_eq!(
            encoded.websocket_url(),
            "ws://127.0.0.1:18080/iot/xiaozhi/ws?protocol_version=3%20beta&device_id=device%20id&client_id=client%2F001&token=token%2Babc%3F"
        );
    }

    #[test]
    fn host_from_base_strips_known_schemes() {
        assert_eq!(host_from_base("http://127.0.0.1:18080"), "127.0.0.1:18080");
        assert_eq!(host_from_base("https://iot.example.com"), "iot.example.com");
        assert_eq!(host_from_base("ws://127.0.0.1:18080"), "127.0.0.1:18080");
        assert_eq!(host_from_base("wss://iot.example.com"), "iot.example.com");
        assert_eq!(host_from_base("gateway.local"), "gateway.local");
        assert_eq!(
            host_from_base("https://iot.example.com/base/path?x=1"),
            "iot.example.com"
        );
    }

    #[test]
    fn hello_message_uses_numeric_protocol_version_with_fallback() {
        let explicit = sample_config("http://127.0.0.1:18080", "9");
        let explicit_message = hello_message(&explicit);
        assert_eq!(explicit_message["type"], json!("hello"));
        assert_eq!(explicit_message["transport"], json!("websocket"));
        assert_eq!(explicit_message["version"], json!(9));
        assert_eq!(explicit_message["features"]["mcp"], json!(true));

        let fallback = sample_config("http://127.0.0.1:18080", "invalid");
        let fallback_message = hello_message(&fallback);
        assert_eq!(fallback_message["version"], json!(3));
    }

    #[test]
    fn session_id_combines_device_and_client() {
        let config = sample_config("http://127.0.0.1:18080", "3");
        assert_eq!(session_id(&config), "device-001-client-001");
    }

    #[test]
    fn app_state_logs_are_bounded_by_capacity() {
        let config = sample_config("http://127.0.0.1:18080", "3");
        let mut app = AppState::new(config);
        for index in 0..(LOG_CAPACITY + 5) {
            app.push_log(format!("line-{index}"));
        }

        assert_eq!(app.logs.len(), LOG_CAPACITY);
        assert!(app.logs.front().is_some());
        assert!(app
            .logs
            .back()
            .expect("latest log must exist")
            .contains("line-204"));
    }
}
