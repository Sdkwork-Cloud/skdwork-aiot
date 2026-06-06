use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rumqttc::{Client, Event, Incoming, MqttOptions, QoS};
use sdkwork_aiot_gateway::WebSocketSessionReply;
use sdkwork_aiot_gateway::XiaozhiMqttUdpSession;

const HTTP_READ_TIMEOUT: Duration = Duration::from_secs(5);
const XIAOZHI_WEBSOCKET_READ_TIMEOUT: Duration = Duration::from_secs(125);
const DEFAULT_MQTT_KEEPALIVE_SECONDS: u64 = 30;
const DEFAULT_MQTT_QUEUE_CAPACITY: usize = 16;
const DEFAULT_MQTT_RECONNECT_BASE_MILLIS: u64 = 500;
const DEFAULT_MQTT_RECONNECT_MAX_MILLIS: u64 = 10_000;
const DEFAULT_SESSION_IDLE_TIMEOUT_SECONDS: u64 = 120;
const DEFAULT_UDP_READ_TIMEOUT_MILLIS: u64 = 2_000;
const DEFAULT_MQTT_PUBLISH_RETRY_ATTEMPTS: u32 = 2;
const DEFAULT_MQTT_PUBLISH_RETRY_DELAY_MILLIS: u64 = 100;
const DEFAULT_BRIDGE_STATS_LOG_INTERVAL_SECONDS: u64 = 30;
const DEFAULT_MQTT_MAX_OUTBOUND_PER_EVENT: usize = 8;
const BRIDGE_STATS_PATH: &str = "/internal/bridge/stats";
const BRIDGE_METRICS_PATH: &str = "/internal/bridge/metrics";
const BRIDGE_HEALTH_PATH: &str = "/internal/bridge/health";
const MCP_POLICY_STATS_PATH: &str = "/internal/xiaozhi/mcp-policy/stats";
const ACTIVATION_REGISTRY_STATS_PATH: &str = "/internal/xiaozhi/activation-registry/stats";
const ACTIVATION_REGISTRY_METRICS_PATH: &str = "/internal/xiaozhi/activation-registry/metrics";

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    setup_shutdown_signal_handler(Arc::clone(&running));

    let (server, session_options) =
        sdkwork_aiot_gateway::standard_gateway_server_and_session_options()
            .expect("gateway transport server");

    println!(
        "sdkwork-aiot-gateway mode={:?} components={} xiaozhi_websocket={}",
        server.runtime.mode(),
        server.runtime.component_names().len(),
        server.runtime.supports_protocol("xiaozhi.websocket")
    );

    if std::env::var("SDKWORK_AIOT_GATEWAY_NO_LISTEN").as_deref() == Ok("1") {
        return;
    }

    let server = Arc::new(server);
    let bridge_enabled =
        std::env::var("SDKWORK_AIOT_GATEWAY_MQTT_BRIDGE_ENABLE").as_deref() == Ok("1");
    let bridge_stats = Arc::new(BridgeStats::new(current_unix_time_millis()));
    let bridge_state = Arc::new(BridgeRuntimeState::new(bridge_enabled));
    if bridge_enabled {
        start_mqtt_udp_bridge(
            Arc::clone(&server),
            session_options.clone(),
            Arc::clone(&running),
            Arc::clone(&bridge_stats),
            Arc::clone(&bridge_state),
        );
    }

    let bind_addr = std::env::var("SDKWORK_AIOT_GATEWAY_BIND")
        .unwrap_or_else(|_| "127.0.0.1:18080".to_string());
    let listener = TcpListener::bind(&bind_addr).expect("bind gateway listener");
    listener
        .set_nonblocking(true)
        .expect("set listener nonblocking");
    println!("sdkwork-aiot-gateway listening on http://{bind_addr}");

    while running.load(Ordering::Relaxed) {
        let stream = match listener.accept() {
            Ok((stream, _peer)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(error) => {
                eprintln!("sdkwork-aiot-gateway accept_error={error}");
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
        };

        let server = Arc::clone(&server);
        let session_options = session_options.clone();
        let bridge_stats = Arc::clone(&bridge_stats);
        let bridge_state = Arc::clone(&bridge_state);
        std::thread::spawn(move || {
            handle_client_connection(
                &server,
                &session_options,
                stream,
                bridge_stats,
                bridge_state,
            )
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MqttBridgeConfig {
    host: String,
    port: u16,
    client_id: String,
    subscribe_topic: String,
    publish_topic: String,
    keep_alive_seconds: u64,
    queue_capacity: usize,
    max_outbound_per_event: usize,
    reconnect_base_delay: Duration,
    reconnect_max_delay: Duration,
    publish_retry_attempts: u32,
    publish_retry_delay: Duration,
    stats_log_interval: Duration,
    publish_drop_cooldown: Duration,
}

impl MqttBridgeConfig {
    fn from_env() -> Self {
        Self {
            host: std::env::var("SDKWORK_AIOT_GATEWAY_MQTT_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env_u16("SDKWORK_AIOT_GATEWAY_MQTT_PORT", 1883),
            client_id: std::env::var("SDKWORK_AIOT_GATEWAY_MQTT_CLIENT_ID")
                .unwrap_or_else(|_| "sdkwork-aiot-gateway".to_string()),
            subscribe_topic: std::env::var("SDKWORK_AIOT_GATEWAY_MQTT_SUBSCRIBE_TOPIC")
                .unwrap_or_else(|_| "xiaozhi/up".to_string()),
            publish_topic: std::env::var("SDKWORK_AIOT_GATEWAY_MQTT_PUBLISH_TOPIC")
                .unwrap_or_else(|_| "xiaozhi/down".to_string()),
            keep_alive_seconds: env_u64(
                "SDKWORK_AIOT_GATEWAY_MQTT_KEEPALIVE_SECONDS",
                DEFAULT_MQTT_KEEPALIVE_SECONDS,
            )
            .max(1),
            queue_capacity: env_usize(
                "SDKWORK_AIOT_GATEWAY_MQTT_QUEUE_CAPACITY",
                DEFAULT_MQTT_QUEUE_CAPACITY,
            )
            .max(1),
            max_outbound_per_event: env_usize(
                "SDKWORK_AIOT_GATEWAY_MQTT_MAX_OUTBOUND_PER_EVENT",
                DEFAULT_MQTT_MAX_OUTBOUND_PER_EVENT,
            )
            .max(1),
            reconnect_base_delay: Duration::from_millis(env_u64(
                "SDKWORK_AIOT_GATEWAY_MQTT_RECONNECT_BASE_MILLIS",
                DEFAULT_MQTT_RECONNECT_BASE_MILLIS,
            )),
            reconnect_max_delay: Duration::from_millis(env_u64(
                "SDKWORK_AIOT_GATEWAY_MQTT_RECONNECT_MAX_MILLIS",
                DEFAULT_MQTT_RECONNECT_MAX_MILLIS,
            )),
            publish_retry_attempts: env_u64(
                "SDKWORK_AIOT_GATEWAY_MQTT_PUBLISH_RETRY_ATTEMPTS",
                DEFAULT_MQTT_PUBLISH_RETRY_ATTEMPTS as u64,
            ) as u32,
            publish_retry_delay: Duration::from_millis(env_u64(
                "SDKWORK_AIOT_GATEWAY_MQTT_PUBLISH_RETRY_DELAY_MILLIS",
                DEFAULT_MQTT_PUBLISH_RETRY_DELAY_MILLIS,
            )),
            stats_log_interval: Duration::from_secs(
                env_u64(
                    "SDKWORK_AIOT_GATEWAY_BRIDGE_STATS_LOG_INTERVAL_SECONDS",
                    DEFAULT_BRIDGE_STATS_LOG_INTERVAL_SECONDS,
                )
                .max(1),
            ),
            publish_drop_cooldown: Duration::from_millis(env_u64(
                "SDKWORK_AIOT_GATEWAY_MQTT_PUBLISH_DROP_COOLDOWN_MILLIS",
                0,
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UdpBridgeConfig {
    bind_addr: String,
    read_timeout: Duration,
    session_idle_timeout: Duration,
    stats_log_interval: Duration,
}

impl UdpBridgeConfig {
    fn from_env() -> Self {
        Self {
            bind_addr: std::env::var("SDKWORK_AIOT_GATEWAY_UDP_BIND")
                .unwrap_or_else(|_| "0.0.0.0:8888".to_string()),
            read_timeout: Duration::from_millis(env_u64(
                "SDKWORK_AIOT_GATEWAY_UDP_READ_TIMEOUT_MILLIS",
                DEFAULT_UDP_READ_TIMEOUT_MILLIS,
            )),
            session_idle_timeout: Duration::from_secs(
                env_u64(
                    "SDKWORK_AIOT_GATEWAY_SESSION_IDLE_TIMEOUT_SECONDS",
                    DEFAULT_SESSION_IDLE_TIMEOUT_SECONDS,
                )
                .max(1),
            ),
            stats_log_interval: Duration::from_secs(
                env_u64(
                    "SDKWORK_AIOT_GATEWAY_BRIDGE_STATS_LOG_INTERVAL_SECONDS",
                    DEFAULT_BRIDGE_STATS_LOG_INTERVAL_SECONDS,
                )
                .max(1),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SessionState {
    session: Option<XiaozhiMqttUdpSession>,
    last_activity_millis: i64,
}

impl SessionState {
    fn new() -> Self {
        Self {
            session: None,
            last_activity_millis: current_unix_time_millis(),
        }
    }

    fn snapshot_session(&self) -> Option<XiaozhiMqttUdpSession> {
        self.session.clone()
    }

    fn touch_now(&mut self) {
        self.last_activity_millis = current_unix_time_millis();
    }

    fn set_session(&mut self, session: Option<XiaozhiMqttUdpSession>) {
        self.session = session;
        self.touch_now();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BridgeStatsSnapshot {
    mqtt_reconnects: u64,
    mqtt_events_total: u64,
    mqtt_events_errors: u64,
    mqtt_session_errors: u64,
    mqtt_publish_attempts: u64,
    mqtt_publish_failures: u64,
    mqtt_publish_retries: u64,
    mqtt_publish_dropped: u64,
    udp_packets_total: u64,
    udp_decode_failures: u64,
    session_idle_purges: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BridgeRuntimeSnapshot {
    bridge_enabled: bool,
    mqtt_loop_running: bool,
    mqtt_session_active: bool,
    udp_loop_running: bool,
    udp_socket_bound: bool,
}

#[derive(Debug)]
struct BridgeRuntimeState {
    bridge_enabled: bool,
    mqtt_loop_running: AtomicBool,
    mqtt_session_active: AtomicBool,
    udp_loop_running: AtomicBool,
    udp_socket_bound: AtomicBool,
}

impl BridgeRuntimeState {
    fn new(bridge_enabled: bool) -> Self {
        Self {
            bridge_enabled,
            mqtt_loop_running: AtomicBool::new(false),
            mqtt_session_active: AtomicBool::new(false),
            udp_loop_running: AtomicBool::new(false),
            udp_socket_bound: AtomicBool::new(false),
        }
    }

    fn snapshot(&self) -> BridgeRuntimeSnapshot {
        BridgeRuntimeSnapshot {
            bridge_enabled: self.bridge_enabled,
            mqtt_loop_running: self.mqtt_loop_running.load(Ordering::Relaxed),
            mqtt_session_active: self.mqtt_session_active.load(Ordering::Relaxed),
            udp_loop_running: self.udp_loop_running.load(Ordering::Relaxed),
            udp_socket_bound: self.udp_socket_bound.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
struct BridgeStats {
    mqtt_reconnects: AtomicU64,
    mqtt_events_total: AtomicU64,
    mqtt_events_errors: AtomicU64,
    mqtt_session_errors: AtomicU64,
    mqtt_publish_attempts: AtomicU64,
    mqtt_publish_failures: AtomicU64,
    mqtt_publish_retries: AtomicU64,
    mqtt_publish_dropped: AtomicU64,
    udp_packets_total: AtomicU64,
    udp_decode_failures: AtomicU64,
    session_idle_purges: AtomicU64,
    last_log_millis: AtomicU64,
}

impl BridgeStats {
    fn new(now_millis: i64) -> Self {
        Self {
            mqtt_reconnects: AtomicU64::new(0),
            mqtt_events_total: AtomicU64::new(0),
            mqtt_events_errors: AtomicU64::new(0),
            mqtt_session_errors: AtomicU64::new(0),
            mqtt_publish_attempts: AtomicU64::new(0),
            mqtt_publish_failures: AtomicU64::new(0),
            mqtt_publish_retries: AtomicU64::new(0),
            mqtt_publish_dropped: AtomicU64::new(0),
            udp_packets_total: AtomicU64::new(0),
            udp_decode_failures: AtomicU64::new(0),
            session_idle_purges: AtomicU64::new(0),
            last_log_millis: AtomicU64::new(now_millis.max(0) as u64),
        }
    }

    fn maybe_log_snapshot(&self, interval: Duration) {
        let now = current_unix_time_millis();
        let now_u64 = now.max(0) as u64;
        let interval_millis = duration_millis_u64(interval).max(1);
        let last = self.last_log_millis.load(Ordering::Relaxed);
        if now_u64.saturating_sub(last) < interval_millis {
            return;
        }
        if self
            .last_log_millis
            .compare_exchange(last, now_u64, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        let snapshot = self.snapshot();
        eprintln!(
            "sdkwork-aiot-gateway bridge_stats reconnects={} mqtt_events={} mqtt_event_errors={} mqtt_session_errors={} publish_attempts={} publish_failures={} publish_retries={} publish_dropped={} udp_packets={} udp_decode_failures={} session_idle_purges={}",
            snapshot.mqtt_reconnects,
            snapshot.mqtt_events_total,
            snapshot.mqtt_events_errors,
            snapshot.mqtt_session_errors,
            snapshot.mqtt_publish_attempts,
            snapshot.mqtt_publish_failures,
            snapshot.mqtt_publish_retries,
            snapshot.mqtt_publish_dropped,
            snapshot.udp_packets_total,
            snapshot.udp_decode_failures,
            snapshot.session_idle_purges,
        );
    }

    fn snapshot(&self) -> BridgeStatsSnapshot {
        BridgeStatsSnapshot {
            mqtt_reconnects: self.mqtt_reconnects.load(Ordering::Relaxed),
            mqtt_events_total: self.mqtt_events_total.load(Ordering::Relaxed),
            mqtt_events_errors: self.mqtt_events_errors.load(Ordering::Relaxed),
            mqtt_session_errors: self.mqtt_session_errors.load(Ordering::Relaxed),
            mqtt_publish_attempts: self.mqtt_publish_attempts.load(Ordering::Relaxed),
            mqtt_publish_failures: self.mqtt_publish_failures.load(Ordering::Relaxed),
            mqtt_publish_retries: self.mqtt_publish_retries.load(Ordering::Relaxed),
            mqtt_publish_dropped: self.mqtt_publish_dropped.load(Ordering::Relaxed),
            udp_packets_total: self.udp_packets_total.load(Ordering::Relaxed),
            udp_decode_failures: self.udp_decode_failures.load(Ordering::Relaxed),
            session_idle_purges: self.session_idle_purges.load(Ordering::Relaxed),
        }
    }
}

fn start_mqtt_udp_bridge(
    server: Arc<sdkwork_aiot_transport::TransportServer>,
    session_options: sdkwork_aiot_gateway::XiaozhiSessionOptions,
    running: Arc<AtomicBool>,
    stats: Arc<BridgeStats>,
    state: Arc<BridgeRuntimeState>,
) {
    let mqtt_config = MqttBridgeConfig::from_env();
    let udp_config = UdpBridgeConfig::from_env();
    let session_state = Arc::new(Mutex::new(SessionState::new()));

    let udp_state = Arc::clone(&session_state);
    let udp_running = Arc::clone(&running);
    let udp_stats = Arc::clone(&stats);
    let udp_runtime = Arc::clone(&state);
    std::thread::spawn(move || {
        run_udp_bridge_loop(udp_state, udp_running, udp_config, udp_stats, udp_runtime)
    });

    let mqtt_state = Arc::clone(&session_state);
    let mqtt_running = Arc::clone(&running);
    let mqtt_stats = Arc::clone(&stats);
    let mqtt_runtime = Arc::clone(&state);
    std::thread::spawn(move || {
        run_mqtt_bridge_loop(
            server,
            session_options,
            mqtt_state,
            mqtt_running,
            mqtt_config,
            mqtt_stats,
            mqtt_runtime,
        );
    });
}

fn run_mqtt_bridge_loop(
    server: Arc<sdkwork_aiot_transport::TransportServer>,
    session_options: sdkwork_aiot_gateway::XiaozhiSessionOptions,
    session_state: Arc<Mutex<SessionState>>,
    running: Arc<AtomicBool>,
    config: MqttBridgeConfig,
    stats: Arc<BridgeStats>,
    state: Arc<BridgeRuntimeState>,
) {
    state.mqtt_loop_running.store(true, Ordering::Relaxed);
    let mut reconnect_attempt = 0u32;
    while running.load(Ordering::Relaxed) {
        if reconnect_attempt > 0 {
            stats.mqtt_reconnects.fetch_add(1, Ordering::Relaxed);
        }
        let mut options =
            MqttOptions::new(config.client_id.clone(), config.host.clone(), config.port);
        options.set_keep_alive(Duration::from_secs(config.keep_alive_seconds));
        let (client, mut connection) = Client::new(options, config.queue_capacity);
        if let Err(error) = client.subscribe(config.subscribe_topic.clone(), QoS::AtMostOnce) {
            eprintln!("sdkwork-aiot-gateway mqtt_subscribe_error={error}");
            state.mqtt_session_active.store(false, Ordering::Relaxed);
            sleep_reconnect_delay(
                reconnect_attempt,
                config.reconnect_base_delay,
                config.reconnect_max_delay,
            );
            reconnect_attempt = reconnect_attempt.saturating_add(1);
            continue;
        }

        reconnect_attempt = 0;
        state.mqtt_session_active.store(true, Ordering::Relaxed);
        loop {
            let event = match connection.iter().next() {
                Some(Ok(event)) => {
                    stats.mqtt_events_total.fetch_add(1, Ordering::Relaxed);
                    event
                }
                Some(Err(error)) => {
                    stats.mqtt_events_errors.fetch_add(1, Ordering::Relaxed);
                    eprintln!("sdkwork-aiot-gateway mqtt_event_error={error}");
                    break;
                }
                None => {
                    eprintln!("sdkwork-aiot-gateway mqtt_event_stream_closed");
                    break;
                }
            };

            if let Event::Incoming(Incoming::Publish(publish)) = event {
                let inbound = String::from_utf8_lossy(&publish.payload).to_string();
                let session_snapshot = {
                    let guard = session_state.lock().expect("mqtt session lock");
                    guard.snapshot_session()
                };

                let response = sdkwork_aiot_gateway::xiaozhi_mqtt_session_reply_with_options(
                    &server,
                    session_snapshot.as_ref(),
                    &inbound,
                    &session_options,
                );
                let (reply, next_session) = match response {
                    Ok(response) => response,
                    Err(error) => {
                        stats.mqtt_session_errors.fetch_add(1, Ordering::Relaxed);
                        eprintln!("sdkwork-aiot-gateway mqtt_session_error={}", error.code);
                        continue;
                    }
                };

                {
                    let mut guard = session_state.lock().expect("mqtt session lock");
                    guard.set_session(next_session);
                }

                let (bounded_outbound, dropped) =
                    bounded_outbound_messages(reply.outbound_json, config.max_outbound_per_event);
                if dropped > 0 {
                    stats
                        .mqtt_publish_dropped
                        .fetch_add(dropped, Ordering::Relaxed);
                }

                for outbound in bounded_outbound {
                    stats.mqtt_publish_attempts.fetch_add(1, Ordering::Relaxed);
                    if let Err(error) = publish_with_retry(
                        &client,
                        &config.publish_topic,
                        outbound,
                        config.publish_retry_attempts,
                        config.publish_retry_delay,
                        stats.as_ref(),
                    ) {
                        eprintln!("sdkwork-aiot-gateway mqtt_publish_error={error}");
                        if duration_millis_u64(config.publish_drop_cooldown) > 0 {
                            std::thread::sleep(config.publish_drop_cooldown);
                        }
                    }
                }
            }
            stats.maybe_log_snapshot(config.stats_log_interval);
        }
        state.mqtt_session_active.store(false, Ordering::Relaxed);

        sleep_reconnect_delay(
            reconnect_attempt,
            config.reconnect_base_delay,
            config.reconnect_max_delay,
        );
        reconnect_attempt = reconnect_attempt.saturating_add(1);
    }
    state.mqtt_session_active.store(false, Ordering::Relaxed);
    state.mqtt_loop_running.store(false, Ordering::Relaxed);
}

fn run_udp_bridge_loop(
    session_state: Arc<Mutex<SessionState>>,
    running: Arc<AtomicBool>,
    config: UdpBridgeConfig,
    stats: Arc<BridgeStats>,
    state: Arc<BridgeRuntimeState>,
) {
    state.udp_loop_running.store(true, Ordering::Relaxed);
    let socket = match UdpSocket::bind(&config.bind_addr) {
        Ok(socket) => socket,
        Err(error) => {
            eprintln!("sdkwork-aiot-gateway udp_bind_error={error}");
            state.udp_socket_bound.store(false, Ordering::Relaxed);
            state.udp_loop_running.store(false, Ordering::Relaxed);
            return;
        }
    };
    state.udp_socket_bound.store(true, Ordering::Relaxed);
    if let Err(error) = socket.set_read_timeout(Some(config.read_timeout)) {
        eprintln!("sdkwork-aiot-gateway udp_timeout_error={error}");
        state.udp_socket_bound.store(false, Ordering::Relaxed);
        state.udp_loop_running.store(false, Ordering::Relaxed);
        return;
    }

    let mut buf = [0u8; 2048];
    while running.load(Ordering::Relaxed) {
        if purge_idle_session(&session_state, config.session_idle_timeout) {
            stats.session_idle_purges.fetch_add(1, Ordering::Relaxed);
        }

        let (len, from) = match socket.recv_from(&mut buf) {
            Ok(value) => value,
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut =>
            {
                continue;
            }
            Err(error) => {
                eprintln!("sdkwork-aiot-gateway udp_recv_error={error}");
                continue;
            }
        };
        stats.udp_packets_total.fetch_add(1, Ordering::Relaxed);

        let mut guard = session_state.lock().expect("udp session lock");
        let Some(session) = guard.session.as_mut() else {
            continue;
        };

        let packet = match sdkwork_aiot_gateway::xiaozhi_mqtt_udp_decode_audio(session, &buf[..len])
        {
            Ok(packet) => packet,
            Err(error) => {
                stats.udp_decode_failures.fetch_add(1, Ordering::Relaxed);
                eprintln!(
                    "sdkwork-aiot-gateway udp_decode_error={} from={from}",
                    error.code
                );
                continue;
            }
        };
        session.remote_sequence = packet.sequence;
        guard.touch_now();
        stats.maybe_log_snapshot(config.stats_log_interval);
    }
    state.udp_socket_bound.store(false, Ordering::Relaxed);
    state.udp_loop_running.store(false, Ordering::Relaxed);
}

fn setup_shutdown_signal_handler(running: Arc<AtomicBool>) {
    install_ctrlc_handler(running);
}

fn install_ctrlc_handler(running: Arc<AtomicBool>) {
    if let Err(error) = ctrlc::set_handler(move || {
        running.store(false, Ordering::SeqCst);
    }) {
        eprintln!("sdkwork-aiot-gateway ctrlc_handler_error={error}");
    }
}

fn handle_client_connection(
    server: &sdkwork_aiot_transport::TransportServer,
    session_options: &sdkwork_aiot_gateway::XiaozhiSessionOptions,
    mut stream: TcpStream,
    bridge_stats: Arc<BridgeStats>,
    bridge_state: Arc<BridgeRuntimeState>,
) {
    let _ = stream.set_read_timeout(Some(HTTP_READ_TIMEOUT));

    let mut buffer = [0u8; 8192];
    let read = match stream.read(&mut buffer) {
        Ok(read) => read,
        Err(error) => {
            eprintln!("sdkwork-aiot-gateway read_error={error}");
            return;
        }
    };
    if read == 0 {
        return;
    }

    let parsed_request = sdkwork_aiot_transport::parse_http_request_prefix(&buffer[..read]);
    if let Ok((request, header_len)) = parsed_request {
        if request.method == "GET" && request.path == BRIDGE_HEALTH_PATH {
            let response = bridge_health_response(bridge_state.as_ref(), bridge_stats.as_ref());
            let response = format_response(&response);
            let _ = stream.write_all(response.as_bytes());
            return;
        }
        if request.method == "GET" && request.path == BRIDGE_STATS_PATH {
            let response = bridge_stats_response(bridge_stats.as_ref());
            let response = format_response(&response);
            let _ = stream.write_all(response.as_bytes());
            return;
        }
        if request.method == "GET" && request.path == BRIDGE_METRICS_PATH {
            let response = bridge_metrics_response(bridge_stats.as_ref());
            let response = format_response(&response);
            let _ = stream.write_all(response.as_bytes());
            return;
        }
        if request.method == "GET" && request.path == MCP_POLICY_STATS_PATH {
            let response = mcp_policy_stats_response(session_options);
            let response = format_response(&response);
            let _ = stream.write_all(response.as_bytes());
            return;
        }
        if request.method == "GET" && request.path == ACTIVATION_REGISTRY_STATS_PATH {
            let response = activation_registry_stats_response();
            let response = format_response(&response);
            let _ = stream.write_all(response.as_bytes());
            return;
        }
        if request.method == "GET" && request.path == ACTIVATION_REGISTRY_METRICS_PATH {
            let response = activation_registry_metrics_response();
            let response = format_response(&response);
            let _ = stream.write_all(response.as_bytes());
            return;
        }

        if is_websocket_upgrade(&request)
            && request.path == sdkwork_aiot_adapter_xiaozhi::XIAOZHI_WS_PATH
        {
            match sdkwork_aiot_transport::build_websocket_handshake_response(&request) {
                Ok(response) => {
                    let response = format_response(&response);
                    if stream.write_all(response.as_bytes()).is_ok() {
                        handle_xiaozhi_websocket_session(
                            &server,
                            session_options,
                            &request,
                            buffer[header_len..read].to_vec(),
                            stream,
                        );
                    }
                }
                Err(error) => {
                    let response = problem_response(&error.code);
                    let _ = stream.write_all(response.as_bytes());
                }
            }
            return;
        }
    }

    let response = match sdkwork_aiot_transport::handle_http_request_bytes(&server, &buffer[..read])
    {
        Ok(response) => response,
        Err(error) => problem_response(&error.code),
    };

    if let Err(error) = stream.write_all(response.as_bytes()) {
        eprintln!("sdkwork-aiot-gateway write_error={error}");
    }
}

fn is_websocket_upgrade(request: &sdkwork_aiot_transport::HttpRequest) -> bool {
    request.method == "GET"
        && request
            .header("upgrade")
            .is_some_and(|value| value.eq_ignore_ascii_case("websocket"))
        && request
            .header("connection")
            .is_some_and(|value| value.to_ascii_lowercase().contains("upgrade"))
}

fn handle_xiaozhi_websocket_session(
    server: &sdkwork_aiot_transport::TransportServer,
    session_options: &sdkwork_aiot_gateway::XiaozhiSessionOptions,
    request: &sdkwork_aiot_transport::HttpRequest,
    initial_frame_bytes: Vec<u8>,
    mut stream: TcpStream,
) {
    let _ = stream.set_read_timeout(Some(XIAOZHI_WEBSOCKET_READ_TIMEOUT));

    let mut read_buffer = [0u8; 8192];
    let mut frame_buffer = initial_frame_bytes;
    loop {
        if !frame_buffer.is_empty()
            && process_xiaozhi_frame_buffer(
                server,
                session_options,
                request,
                &mut stream,
                &mut frame_buffer,
            )
        {
            return;
        }

        let read = match stream.read(&mut read_buffer) {
            Ok(0) => return,
            Ok(read) => read,
            Err(error) => {
                eprintln!("sdkwork-aiot-gateway websocket_read_error={error}");
                return;
            }
        };
        frame_buffer.extend_from_slice(&read_buffer[..read]);
    }
}

fn process_xiaozhi_frame_buffer(
    server: &sdkwork_aiot_transport::TransportServer,
    session_options: &sdkwork_aiot_gateway::XiaozhiSessionOptions,
    request: &sdkwork_aiot_transport::HttpRequest,
    stream: &mut TcpStream,
    frame_buffer: &mut Vec<u8>,
) -> bool {
    loop {
        let (frame, used) =
            match sdkwork_aiot_transport::decode_websocket_frame_prefix(frame_buffer) {
                Ok(result) => result,
                Err(error) if error.code == "transport.websocket.incomplete_frame" => return false,
                Err(error) => {
                    eprintln!("sdkwork-aiot-gateway websocket_decode_error={}", error.code);
                    return true;
                }
            };
        frame_buffer.drain(..used);

        let replies = match sdkwork_aiot_gateway::xiaozhi_websocket_session_reply_with_options(
            server,
            request,
            frame,
            session_options,
        ) {
            Ok(replies) => replies,
            Err(error) => {
                vec![WebSocketSessionReply::Text(format!(
                    r#"{{"type":"alert","status":"Bad Request","message":"{}","emotion":"sad"}}"#,
                    json_escape(&error.code)
                ))]
            }
        };

        for reply in replies {
            let should_close = matches!(reply, WebSocketSessionReply::Close);
            let frame = websocket_reply_frame(reply);
            if let Err(error) = stream.write_all(&frame) {
                eprintln!("sdkwork-aiot-gateway websocket_write_error={error}");
                return true;
            }
            if should_close {
                return true;
            }
        }
    }
}

fn websocket_reply_frame(reply: WebSocketSessionReply) -> Vec<u8> {
    let frame = match reply {
        WebSocketSessionReply::Text(text) => sdkwork_aiot_transport::WebSocketFrame::text(text),
        WebSocketSessionReply::Binary(payload) => sdkwork_aiot_transport::WebSocketFrame {
            opcode: sdkwork_aiot_transport::WebSocketOpcode::Binary,
            payload,
        },
        WebSocketSessionReply::Pong(payload) => sdkwork_aiot_transport::WebSocketFrame {
            opcode: sdkwork_aiot_transport::WebSocketOpcode::Pong,
            payload,
        },
        WebSocketSessionReply::Close => sdkwork_aiot_transport::WebSocketFrame {
            opcode: sdkwork_aiot_transport::WebSocketOpcode::Close,
            payload: Vec::new(),
        },
    };

    sdkwork_aiot_transport::encode_websocket_frame(&frame)
}

fn format_response(response: &sdkwork_aiot_transport::HttpResponse) -> String {
    let mut out = format!(
        "HTTP/1.1 {} {}\r\n",
        response.status.code(),
        response.status.reason()
    );
    let mut has_content_length = false;
    for (name, value) in response.headers() {
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

fn bridge_health_response(
    runtime_state: &BridgeRuntimeState,
    stats: &BridgeStats,
) -> sdkwork_aiot_transport::HttpResponse {
    let runtime_snapshot = runtime_state.snapshot();
    let stats_snapshot = stats.snapshot();
    let body = bridge_health_snapshot_json(&runtime_snapshot, &stats_snapshot);
    sdkwork_aiot_transport::HttpResponse::new(sdkwork_aiot_transport::HttpStatus::Ok)
        .with_header("content-type", "application/json")
        .with_body(body)
}

fn bridge_stats_response(stats: &BridgeStats) -> sdkwork_aiot_transport::HttpResponse {
    let snapshot = stats.snapshot();
    let body = bridge_stats_snapshot_json(&snapshot);
    sdkwork_aiot_transport::HttpResponse::new(sdkwork_aiot_transport::HttpStatus::Ok)
        .with_header("content-type", "application/json")
        .with_body(body)
}

fn bridge_metrics_response(stats: &BridgeStats) -> sdkwork_aiot_transport::HttpResponse {
    let snapshot = stats.snapshot();
    let activation_registry_snapshot =
        sdkwork_aiot_gateway::xiaozhi_activation_registry_stats_snapshot();
    let mut body = bridge_stats_prometheus_text(&snapshot);
    body.push_str(&activation_registry_prometheus_text(
        &activation_registry_snapshot,
    ));
    sdkwork_aiot_transport::HttpResponse::new(sdkwork_aiot_transport::HttpStatus::Ok)
        .with_header("content-type", "text/plain; version=0.0.4")
        .with_body(body)
}

fn mcp_policy_stats_response(
    session_options: &sdkwork_aiot_gateway::XiaozhiSessionOptions,
) -> sdkwork_aiot_transport::HttpResponse {
    let policy = session_options.mcp_tool_policy();
    let body = if let Some(snapshot) = policy.stats_snapshot() {
        format!(
            r#"{{"policy":"rule_based","allow_by_rule_matches":{},"allow_no_rule_matches":{},"deny_by_rule_matches":{}}}"#,
            snapshot.allow_by_rule_matches,
            snapshot.allow_no_rule_matches,
            snapshot.deny_by_rule_matches
        )
    } else {
        r#"{"policy":"custom","stats_available":false}"#.to_string()
    };
    sdkwork_aiot_transport::HttpResponse::new(sdkwork_aiot_transport::HttpStatus::Ok)
        .with_header("content-type", "application/json")
        .with_body(body)
}

fn activation_registry_stats_response() -> sdkwork_aiot_transport::HttpResponse {
    let snapshot = sdkwork_aiot_gateway::xiaozhi_activation_registry_stats_snapshot();
    let body = format!(
        r#"{{"backend":"{}","register_total":{},"consume_total":{},"consume_hits":{},"consume_misses":{},"pruned_entries":{}}}"#,
        snapshot.backend_kind,
        snapshot.register_total,
        snapshot.consume_total,
        snapshot.consume_hits,
        snapshot.consume_misses,
        snapshot.pruned_entries
    );
    sdkwork_aiot_transport::HttpResponse::new(sdkwork_aiot_transport::HttpStatus::Ok)
        .with_header("content-type", "application/json")
        .with_body(body)
}

fn activation_registry_metrics_response() -> sdkwork_aiot_transport::HttpResponse {
    let snapshot = sdkwork_aiot_gateway::xiaozhi_activation_registry_stats_snapshot();
    let body = activation_registry_prometheus_text(&snapshot);
    sdkwork_aiot_transport::HttpResponse::new(sdkwork_aiot_transport::HttpStatus::Ok)
        .with_header("content-type", "text/plain; version=0.0.4")
        .with_body(body)
}

fn activation_registry_prometheus_text(
    snapshot: &sdkwork_aiot_gateway::XiaozhiActivationRegistryStatsSnapshot,
) -> String {
    let backend = json_escape(&snapshot.backend_kind);
    format!(
        "sdkwork_aiot_xiaozhi_activation_registry_register_total {}\n\
sdkwork_aiot_xiaozhi_activation_registry_consume_total {}\n\
sdkwork_aiot_xiaozhi_activation_registry_consume_hits_total {}\n\
sdkwork_aiot_xiaozhi_activation_registry_consume_misses_total {}\n\
sdkwork_aiot_xiaozhi_activation_registry_pruned_entries_total {}\n\
sdkwork_aiot_xiaozhi_activation_registry_backend{{backend=\"{}\"}} 1\n",
        snapshot.register_total,
        snapshot.consume_total,
        snapshot.consume_hits,
        snapshot.consume_misses,
        snapshot.pruned_entries,
        backend
    )
}

fn bridge_health_snapshot_json(
    runtime: &BridgeRuntimeSnapshot,
    stats: &BridgeStatsSnapshot,
) -> String {
    let status = if !runtime.bridge_enabled {
        "disabled"
    } else if runtime.mqtt_loop_running && runtime.udp_loop_running && runtime.udp_socket_bound {
        "ok"
    } else {
        "degraded"
    };
    let stats_json = bridge_stats_snapshot_json(stats);
    format!(
        r#"{{"status":"{}","bridge_enabled":{},"mqtt":{{"loop_running":{},"session_active":{}}},"udp":{{"loop_running":{},"socket_bound":{}}},"stats":{}}}"#,
        status,
        runtime.bridge_enabled,
        runtime.mqtt_loop_running,
        runtime.mqtt_session_active,
        runtime.udp_loop_running,
        runtime.udp_socket_bound,
        stats_json
    )
}

fn bridge_stats_snapshot_json(snapshot: &BridgeStatsSnapshot) -> String {
    format!(
        r#"{{"mqtt_reconnects":{},"mqtt_events_total":{},"mqtt_events_errors":{},"mqtt_session_errors":{},"mqtt_publish_attempts":{},"mqtt_publish_failures":{},"mqtt_publish_retries":{},"mqtt_publish_dropped":{},"udp_packets_total":{},"udp_decode_failures":{},"session_idle_purges":{}}}"#,
        snapshot.mqtt_reconnects,
        snapshot.mqtt_events_total,
        snapshot.mqtt_events_errors,
        snapshot.mqtt_session_errors,
        snapshot.mqtt_publish_attempts,
        snapshot.mqtt_publish_failures,
        snapshot.mqtt_publish_retries,
        snapshot.mqtt_publish_dropped,
        snapshot.udp_packets_total,
        snapshot.udp_decode_failures,
        snapshot.session_idle_purges
    )
}

fn bridge_stats_prometheus_text(snapshot: &BridgeStatsSnapshot) -> String {
    let lines = [
        (
            "sdkwork_aiot_bridge_mqtt_reconnects_total",
            snapshot.mqtt_reconnects,
        ),
        (
            "sdkwork_aiot_bridge_mqtt_events_total",
            snapshot.mqtt_events_total,
        ),
        (
            "sdkwork_aiot_bridge_mqtt_event_errors_total",
            snapshot.mqtt_events_errors,
        ),
        (
            "sdkwork_aiot_bridge_mqtt_session_errors_total",
            snapshot.mqtt_session_errors,
        ),
        (
            "sdkwork_aiot_bridge_mqtt_publish_attempts_total",
            snapshot.mqtt_publish_attempts,
        ),
        (
            "sdkwork_aiot_bridge_mqtt_publish_failures_total",
            snapshot.mqtt_publish_failures,
        ),
        (
            "sdkwork_aiot_bridge_mqtt_publish_retries_total",
            snapshot.mqtt_publish_retries,
        ),
        (
            "sdkwork_aiot_bridge_mqtt_publish_dropped_total",
            snapshot.mqtt_publish_dropped,
        ),
        (
            "sdkwork_aiot_bridge_udp_packets_total",
            snapshot.udp_packets_total,
        ),
        (
            "sdkwork_aiot_bridge_udp_decode_failures_total",
            snapshot.udp_decode_failures,
        ),
        (
            "sdkwork_aiot_bridge_session_idle_purges_total",
            snapshot.session_idle_purges,
        ),
    ];

    let mut out = String::new();
    for (metric, value) in lines {
        out.push_str(metric);
        out.push(' ');
        out.push_str(&value.to_string());
        out.push('\n');
    }
    out
}

fn problem_response(code: &str) -> String {
    let body = format!(
        r#"{{"type":"about:blank","title":"Bad Request","status":400,"code":"{}"}}"#,
        json_escape(code)
    );
    format!(
        "HTTP/1.1 400 Bad Request\r\ncontent-type: application/problem+json\r\ncontent-length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

fn json_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch < ' ' => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

fn sleep_reconnect_delay(attempt: u32, base: Duration, max: Duration) {
    let delay = reconnect_delay(attempt, base, max);
    std::thread::sleep(delay);
}

fn publish_with_retry(
    client: &Client,
    topic: &str,
    payload: String,
    retry_attempts: u32,
    retry_delay: Duration,
    stats: &BridgeStats,
) -> Result<(), String> {
    let mut attempts = 0u32;
    loop {
        match client.publish(topic, QoS::AtMostOnce, false, payload.clone()) {
            Ok(()) => return Ok(()),
            Err(error) => {
                if attempts >= retry_attempts {
                    stats.mqtt_publish_failures.fetch_add(1, Ordering::Relaxed);
                    return Err(error.to_string());
                }
                attempts = attempts.saturating_add(1);
                stats.mqtt_publish_retries.fetch_add(1, Ordering::Relaxed);
                std::thread::sleep(retry_delay);
            }
        }
    }
}

fn bounded_outbound_messages(messages: Vec<String>, max_outbound: usize) -> (Vec<String>, u64) {
    let limit = max_outbound.max(1);
    let dropped = messages.len().saturating_sub(limit) as u64;
    let bounded = messages.into_iter().take(limit).collect::<Vec<_>>();
    (bounded, dropped)
}

fn reconnect_delay(attempt: u32, base: Duration, max: Duration) -> Duration {
    let base_millis = duration_millis_u64(base).max(1);
    let max_millis = duration_millis_u64(max).max(base_millis);
    let shift = attempt.min(20);
    let factor = 1u64 << shift;
    let expanded = base_millis.saturating_mul(factor);
    Duration::from_millis(expanded.min(max_millis))
}

fn purge_idle_session(
    session_state: &Arc<Mutex<SessionState>>,
    session_idle_timeout: Duration,
) -> bool {
    let now = current_unix_time_millis();
    let idle_timeout_millis = duration_millis_i64(session_idle_timeout).max(1);
    let mut guard = session_state.lock().expect("udp session lock");
    if should_purge_idle_session(now, guard.last_activity_millis, idle_timeout_millis) {
        guard.session = None;
        return true;
    }
    false
}

fn should_purge_idle_session(
    now_millis: i64,
    last_activity_millis: i64,
    idle_timeout_millis: i64,
) -> bool {
    now_millis.saturating_sub(last_activity_millis) >= idle_timeout_millis
}

fn current_unix_time_millis() -> i64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

fn duration_millis_u64(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn duration_millis_i64(duration: Duration) -> i64 {
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

fn env_string(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_u64(name: &str, default: u64) -> u64 {
    env_string(name)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    env_string(name)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn env_u16(name: &str, default: u16) -> u16 {
    env_string(name)
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconnect_delay_uses_exponential_backoff_and_cap() {
        let base = Duration::from_millis(500);
        let max = Duration::from_millis(10_000);

        assert_eq!(reconnect_delay(0, base, max), Duration::from_millis(500));
        assert_eq!(reconnect_delay(1, base, max), Duration::from_millis(1_000));
        assert_eq!(reconnect_delay(3, base, max), Duration::from_millis(4_000));
        assert_eq!(
            reconnect_delay(10, base, max),
            Duration::from_millis(10_000)
        );
    }

    #[test]
    fn reconnect_delay_never_returns_zero() {
        let delay = reconnect_delay(0, Duration::from_millis(0), Duration::from_millis(0));
        assert_eq!(delay, Duration::from_millis(1));
    }

    #[test]
    fn should_purge_idle_session_detects_timeout_threshold() {
        assert!(!should_purge_idle_session(10_000, 9_001, 1_000));
        assert!(should_purge_idle_session(10_000, 9_000, 1_000));
        assert!(should_purge_idle_session(10_000, 8_500, 1_000));
    }

    #[test]
    fn purge_idle_session_clears_session_when_timed_out() {
        let session = XiaozhiMqttUdpSession {
            device_id: "dev-001".to_string(),
            client_id: "client-001".to_string(),
            session_id: "dev-001-client-001".to_string(),
            udp_server: "127.0.0.1".to_string(),
            udp_port: 8888,
            udp_key_hex: "00112233445566778899AABBCCDDEEFF".to_string(),
            udp_nonce_hex: "01000000A1A2A3A40000000000000000".to_string(),
            remote_sequence: 42,
        };
        let state = Arc::new(Mutex::new(SessionState {
            session: Some(session),
            last_activity_millis: 0,
        }));

        let purged = purge_idle_session(&state, Duration::from_millis(1));
        assert!(purged);
        let guard = state.lock().expect("state lock");
        assert!(guard.session.is_none());
    }

    #[test]
    fn bridge_stats_snapshot_json_contains_all_fields() {
        let snapshot = BridgeStatsSnapshot {
            mqtt_reconnects: 1,
            mqtt_events_total: 2,
            mqtt_events_errors: 3,
            mqtt_session_errors: 4,
            mqtt_publish_attempts: 5,
            mqtt_publish_failures: 6,
            mqtt_publish_retries: 7,
            mqtt_publish_dropped: 8,
            udp_packets_total: 9,
            udp_decode_failures: 10,
            session_idle_purges: 11,
        };

        let json = bridge_stats_snapshot_json(&snapshot);
        assert!(json.contains(r#""mqtt_reconnects":1"#));
        assert!(json.contains(r#""mqtt_events_total":2"#));
        assert!(json.contains(r#""mqtt_events_errors":3"#));
        assert!(json.contains(r#""mqtt_session_errors":4"#));
        assert!(json.contains(r#""mqtt_publish_attempts":5"#));
        assert!(json.contains(r#""mqtt_publish_failures":6"#));
        assert!(json.contains(r#""mqtt_publish_retries":7"#));
        assert!(json.contains(r#""mqtt_publish_dropped":8"#));
        assert!(json.contains(r#""udp_packets_total":9"#));
        assert!(json.contains(r#""udp_decode_failures":10"#));
        assert!(json.contains(r#""session_idle_purges":11"#));
    }

    #[test]
    fn bridge_stats_prometheus_text_contains_expected_metrics() {
        let snapshot = BridgeStatsSnapshot {
            mqtt_reconnects: 1,
            mqtt_events_total: 2,
            mqtt_events_errors: 3,
            mqtt_session_errors: 4,
            mqtt_publish_attempts: 5,
            mqtt_publish_failures: 6,
            mqtt_publish_retries: 7,
            mqtt_publish_dropped: 8,
            udp_packets_total: 9,
            udp_decode_failures: 10,
            session_idle_purges: 11,
        };

        let text = bridge_stats_prometheus_text(&snapshot);
        assert!(text.contains("sdkwork_aiot_bridge_mqtt_reconnects_total 1"));
        assert!(text.contains("sdkwork_aiot_bridge_mqtt_events_total 2"));
        assert!(text.contains("sdkwork_aiot_bridge_mqtt_event_errors_total 3"));
        assert!(text.contains("sdkwork_aiot_bridge_mqtt_session_errors_total 4"));
        assert!(text.contains("sdkwork_aiot_bridge_mqtt_publish_attempts_total 5"));
        assert!(text.contains("sdkwork_aiot_bridge_mqtt_publish_failures_total 6"));
        assert!(text.contains("sdkwork_aiot_bridge_mqtt_publish_retries_total 7"));
        assert!(text.contains("sdkwork_aiot_bridge_mqtt_publish_dropped_total 8"));
        assert!(text.contains("sdkwork_aiot_bridge_udp_packets_total 9"));
        assert!(text.contains("sdkwork_aiot_bridge_udp_decode_failures_total 10"));
        assert!(text.contains("sdkwork_aiot_bridge_session_idle_purges_total 11"));
    }

    #[test]
    fn bridge_health_snapshot_json_reports_runtime_status_and_stats() {
        let runtime = BridgeRuntimeSnapshot {
            bridge_enabled: true,
            mqtt_loop_running: true,
            mqtt_session_active: true,
            udp_loop_running: true,
            udp_socket_bound: true,
        };
        let stats = BridgeStatsSnapshot {
            mqtt_reconnects: 1,
            mqtt_events_total: 2,
            mqtt_events_errors: 3,
            mqtt_session_errors: 4,
            mqtt_publish_attempts: 5,
            mqtt_publish_failures: 6,
            mqtt_publish_retries: 7,
            mqtt_publish_dropped: 8,
            udp_packets_total: 9,
            udp_decode_failures: 10,
            session_idle_purges: 11,
        };

        let json = bridge_health_snapshot_json(&runtime, &stats);
        assert!(json.contains(r#""status":"ok""#));
        assert!(json.contains(r#""bridge_enabled":true"#));
        assert!(json.contains(r#""mqtt":{"loop_running":true,"session_active":true}"#));
        assert!(json.contains(r#""udp":{"loop_running":true,"socket_bound":true}"#));
        assert!(json.contains(r#""stats":{"mqtt_reconnects":1"#));
    }

    #[test]
    fn bridge_health_snapshot_json_reports_disabled_when_bridge_is_off() {
        let runtime = BridgeRuntimeSnapshot {
            bridge_enabled: false,
            mqtt_loop_running: false,
            mqtt_session_active: false,
            udp_loop_running: false,
            udp_socket_bound: false,
        };
        let stats = BridgeStatsSnapshot {
            mqtt_reconnects: 0,
            mqtt_events_total: 0,
            mqtt_events_errors: 0,
            mqtt_session_errors: 0,
            mqtt_publish_attempts: 0,
            mqtt_publish_failures: 0,
            mqtt_publish_retries: 0,
            mqtt_publish_dropped: 0,
            udp_packets_total: 0,
            udp_decode_failures: 0,
            session_idle_purges: 0,
        };

        let json = bridge_health_snapshot_json(&runtime, &stats);
        assert!(json.contains(r#""status":"disabled""#));
        assert!(json.contains(r#""bridge_enabled":false"#));
    }

    #[test]
    fn bounded_outbound_messages_limits_payload_and_reports_dropped_count() {
        let messages = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let (bounded, dropped) = bounded_outbound_messages(messages, 2);
        assert_eq!(bounded, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(dropped, 2);
    }
}
