use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sdkwork_aiot_adapter_xiaozhi::{
    xiaozhi_activation_accepted_response, xiaozhi_activation_pending_response,
    xiaozhi_handshake_context, xiaozhi_ota_response, xiaozhi_server_hello_response,
    XiaozhiAudioParams, XiaozhiMqttCodec, XiaozhiOtaMetadata, XiaozhiServerHello,
    XiaozhiUdpAudioCodec, XiaozhiUdpAudioPacket, XiaozhiWebSocketCodec, AUTHORIZATION_HEADER,
    CLIENT_ID_HEADER, DEVICE_ID_HEADER, PROTOCOL_VERSION_HEADER, XIAOZHI_ACTIVATE_PATH,
    XIAOZHI_MQTT_PATH, XIAOZHI_OTA_ACTIVATE_PATH, XIAOZHI_OTA_PATH, XIAOZHI_WS_PATH,
};
use sdkwork_aiot_transport::{
    websocket_frame_to_inbound_frame, HttpRequest, HttpResponse, HttpStatus, TransportError,
    TransportServer, WebSocketFrame, WebSocketOpcode,
};

thread_local! {
    static SQLITE_CONNECTION_REGISTRY: RefCell<HashMap<PathBuf, Arc<Mutex<rusqlite::Connection>>>> =
        RefCell::new(HashMap::new());
}

pub const XIAOZHI_SIMULATOR_PATH: &str = "/simulators/xiaozhi";
const DEFAULT_DEVICE_TOKEN: &str = "device-token";
const DEFAULT_ACTIVATION_MESSAGE: &str = "activation pending";
const DEFAULT_ACTIVATION_TIMEOUT_MS: u32 = 30_000;
const DEFAULT_ACTIVATION_REGISTRY_LOCK_WAIT_MILLIS: u64 = 2_000;
const DEFAULT_ACTIVATION_REGISTRY_LOCK_POLL_MILLIS: u64 = 20;
const DEFAULT_ACTIVATION_REGISTRY_LOCK_STALE_MILLIS: u64 = 30_000;
const DEFAULT_MQTT_KEEPALIVE_SECONDS: u32 = 240;
const DEFAULT_SERVER_TIMEZONE_OFFSET_MINUTES: i32 = 480;
const DEFAULT_SIMULATOR_MCP_PAGE_SIZE: usize = 2;
const SIMULATOR_SERVER_NAME: &str = "sdkwork-aiot-gateway";
const SIMULATOR_PROTOCOL_VERSION: &str = "2024-11-05";
const ENV_XIAOZHI_DEVICE_TOKEN: &str = "SDKWORK_AIOT_XIAOZHI_DEVICE_TOKEN";
const ENV_XIAOZHI_MQTT_ENDPOINT: &str = "SDKWORK_AIOT_XIAOZHI_MQTT_ENDPOINT";
const ENV_XIAOZHI_MQTT_CLIENT_ID: &str = "SDKWORK_AIOT_XIAOZHI_MQTT_CLIENT_ID";
const ENV_XIAOZHI_MQTT_USERNAME: &str = "SDKWORK_AIOT_XIAOZHI_MQTT_USERNAME";
const ENV_XIAOZHI_MQTT_PASSWORD: &str = "SDKWORK_AIOT_XIAOZHI_MQTT_PASSWORD";
const ENV_XIAOZHI_MQTT_PUBLISH_TOPIC: &str = "SDKWORK_AIOT_XIAOZHI_MQTT_PUBLISH_TOPIC";
const ENV_XIAOZHI_MQTT_SUBSCRIBE_TOPIC: &str = "SDKWORK_AIOT_XIAOZHI_MQTT_SUBSCRIBE_TOPIC";
const ENV_XIAOZHI_MQTT_KEEPALIVE: &str = "SDKWORK_AIOT_XIAOZHI_MQTT_KEEPALIVE_SECONDS";
const ENV_XIAOZHI_MQTT_UDP_SERVER: &str = "SDKWORK_AIOT_XIAOZHI_MQTT_UDP_SERVER";
const ENV_XIAOZHI_MQTT_UDP_PORT: &str = "SDKWORK_AIOT_XIAOZHI_MQTT_UDP_PORT";
const ENV_XIAOZHI_MQTT_UDP_KEY_HEX: &str = "SDKWORK_AIOT_XIAOZHI_MQTT_UDP_KEY_HEX";
const ENV_XIAOZHI_MQTT_UDP_NONCE_HEX: &str = "SDKWORK_AIOT_XIAOZHI_MQTT_UDP_NONCE_HEX";
const ENV_XIAOZHI_FIRMWARE_VERSION: &str = "SDKWORK_AIOT_XIAOZHI_FIRMWARE_VERSION";
const ENV_XIAOZHI_FIRMWARE_URL: &str = "SDKWORK_AIOT_XIAOZHI_FIRMWARE_URL";
const ENV_XIAOZHI_FIRMWARE_FORCE: &str = "SDKWORK_AIOT_XIAOZHI_FIRMWARE_FORCE";
const ENV_XIAOZHI_ACTIVATION_MESSAGE: &str = "SDKWORK_AIOT_XIAOZHI_ACTIVATION_MESSAGE";
const ENV_XIAOZHI_ACTIVATION_CODE: &str = "SDKWORK_AIOT_XIAOZHI_ACTIVATION_CODE";
const ENV_XIAOZHI_ACTIVATION_CHALLENGE: &str = "SDKWORK_AIOT_XIAOZHI_ACTIVATION_CHALLENGE";
const ENV_XIAOZHI_ACTIVATION_TIMEOUT_MS: &str = "SDKWORK_AIOT_XIAOZHI_ACTIVATION_TIMEOUT_MS";
const ENV_XIAOZHI_ACTIVATE_AUTO_ACCEPT: &str = "SDKWORK_AIOT_XIAOZHI_ACTIVATE_AUTO_ACCEPT";
const ENV_XIAOZHI_ACTIVATE_EXPECTED_CHALLENGE: &str =
    "SDKWORK_AIOT_XIAOZHI_ACTIVATE_EXPECTED_CHALLENGE";
const ENV_XIAOZHI_ACTIVATE_EXPECTED_HMAC: &str = "SDKWORK_AIOT_XIAOZHI_ACTIVATE_EXPECTED_HMAC";
const ENV_XIAOZHI_ACTIVATE_STRICT_V2: &str = "SDKWORK_AIOT_XIAOZHI_ACTIVATE_STRICT_V2";
const ENV_XIAOZHI_ACTIVATION_REGISTRY_PATH: &str = "SDKWORK_AIOT_XIAOZHI_ACTIVATION_REGISTRY_PATH";
const ENV_XIAOZHI_ACTIVATION_REGISTRY_KIND: &str = "SDKWORK_AIOT_XIAOZHI_ACTIVATION_REGISTRY_KIND";
const ENV_XIAOZHI_ACTIVATION_REGISTRY_LOCK_WAIT_MILLIS: &str =
    "SDKWORK_AIOT_XIAOZHI_ACTIVATION_REGISTRY_LOCK_WAIT_MILLIS";
const ENV_XIAOZHI_ACTIVATION_REGISTRY_LOCK_POLL_MILLIS: &str =
    "SDKWORK_AIOT_XIAOZHI_ACTIVATION_REGISTRY_LOCK_POLL_MILLIS";
const ENV_XIAOZHI_ACTIVATION_REGISTRY_LOCK_STALE_MILLIS: &str =
    "SDKWORK_AIOT_XIAOZHI_ACTIVATION_REGISTRY_LOCK_STALE_MILLIS";
const ENV_XIAOZHI_ACTIVATION_REGISTRY_REDIS_URL: &str =
    "SDKWORK_AIOT_XIAOZHI_ACTIVATION_REGISTRY_REDIS_URL";
const ENV_XIAOZHI_ACTIVATION_REGISTRY_REDIS_PREFIX: &str =
    "SDKWORK_AIOT_XIAOZHI_ACTIVATION_REGISTRY_REDIS_PREFIX";
const ENV_XIAOZHI_SIMULATOR_MCP_TOOLS_PATH: &str = "SDKWORK_AIOT_XIAOZHI_SIMULATOR_MCP_TOOLS_PATH";
const ENV_XIAOZHI_MCP_POLICY_RULES: &str = "SDKWORK_AIOT_XIAOZHI_MCP_POLICY_RULES";
const ENV_XIAOZHI_MCP_POLICY_LOG_ALLOW: &str = "SDKWORK_AIOT_XIAOZHI_MCP_POLICY_LOG_ALLOW";
const ENV_XIAOZHI_SERVER_TIMEZONE_OFFSET_MINUTES: &str =
    "SDKWORK_AIOT_XIAOZHI_SERVER_TIMEZONE_OFFSET_MINUTES";

const ACTIVATION_REGISTRY_BACKEND_UNKNOWN: u64 = 0;
const ACTIVATION_REGISTRY_BACKEND_IN_MEMORY: u64 = 1;
const ACTIVATION_REGISTRY_BACKEND_FILE: u64 = 2;
const ACTIVATION_REGISTRY_BACKEND_SQLITE: u64 = 3;
const ACTIVATION_REGISTRY_BACKEND_REDIS: u64 = 4;

#[derive(Debug)]
struct ActivationRegistryStats {
    backend_kind: AtomicU64,
    register_total: AtomicU64,
    consume_total: AtomicU64,
    consume_hits: AtomicU64,
    consume_misses: AtomicU64,
    pruned_entries: AtomicU64,
}

static ACTIVATION_REGISTRY_STATS: ActivationRegistryStats = ActivationRegistryStats {
    backend_kind: AtomicU64::new(ACTIVATION_REGISTRY_BACKEND_UNKNOWN),
    register_total: AtomicU64::new(0),
    consume_total: AtomicU64::new(0),
    consume_hits: AtomicU64::new(0),
    consume_misses: AtomicU64::new(0),
    pruned_entries: AtomicU64::new(0),
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XiaozhiActivationRegistryStatsSnapshot {
    pub backend_kind: String,
    pub register_total: u64,
    pub consume_total: u64,
    pub consume_hits: u64,
    pub consume_misses: u64,
    pub pruned_entries: u64,
}

pub fn xiaozhi_activation_registry_stats_snapshot() -> XiaozhiActivationRegistryStatsSnapshot {
    let backend_code = ACTIVATION_REGISTRY_STATS
        .backend_kind
        .load(Ordering::Relaxed);
    XiaozhiActivationRegistryStatsSnapshot {
        backend_kind: activation_registry_backend_name(backend_code).to_string(),
        register_total: ACTIVATION_REGISTRY_STATS
            .register_total
            .load(Ordering::Relaxed),
        consume_total: ACTIVATION_REGISTRY_STATS
            .consume_total
            .load(Ordering::Relaxed),
        consume_hits: ACTIVATION_REGISTRY_STATS
            .consume_hits
            .load(Ordering::Relaxed),
        consume_misses: ACTIVATION_REGISTRY_STATS
            .consume_misses
            .load(Ordering::Relaxed),
        pruned_entries: ACTIVATION_REGISTRY_STATS
            .pruned_entries
            .load(Ordering::Relaxed),
    }
}

pub trait XiaozhiActivationVerifier: Send + Sync {
    fn is_accepted(&self, request: &HttpRequest) -> bool;
}

pub trait XiaozhiActivationChallengeRegistry: Send + Sync {
    fn register_challenge(&self, request: &HttpRequest, challenge: &str, timeout_ms: u32);
    fn consume_challenge(&self, request: &HttpRequest, challenge: &str) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ActivationChallengeKey {
    device_id: String,
    client_id: String,
    challenge: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActivationChallengeEntry {
    expires_at_millis: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActivationRegistryRecord {
    key: ActivationChallengeKey,
    entry: ActivationChallengeEntry,
}

#[derive(Debug, Clone)]
pub struct InMemoryXiaozhiActivationChallengeRegistry {
    entries: Arc<Mutex<HashMap<ActivationChallengeKey, ActivationChallengeEntry>>>,
}

impl InMemoryXiaozhiActivationChallengeRegistry {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryXiaozhiActivationChallengeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl XiaozhiActivationChallengeRegistry for InMemoryXiaozhiActivationChallengeRegistry {
    fn register_challenge(&self, request: &HttpRequest, challenge: &str, timeout_ms: u32) {
        activation_registry_mark_backend(ACTIVATION_REGISTRY_BACKEND_IN_MEMORY);
        ACTIVATION_REGISTRY_STATS
            .register_total
            .fetch_add(1, Ordering::Relaxed);
        let mut entries = self
            .entries
            .lock()
            .expect("xiaozhi activation challenge registry lock");
        let pruned = register_challenge_in_entries(&mut entries, request, challenge, timeout_ms);
        activation_registry_add_pruned(pruned);
    }

    fn consume_challenge(&self, request: &HttpRequest, challenge: &str) -> bool {
        activation_registry_mark_backend(ACTIVATION_REGISTRY_BACKEND_IN_MEMORY);
        ACTIVATION_REGISTRY_STATS
            .consume_total
            .fetch_add(1, Ordering::Relaxed);
        let mut entries = self
            .entries
            .lock()
            .expect("xiaozhi activation challenge registry lock");
        let (consumed, pruned) = consume_challenge_in_entries(&mut entries, request, challenge);
        activation_registry_add_pruned(pruned);
        activation_registry_record_consume_outcome(consumed);
        consumed
    }
}

#[derive(Debug, Clone)]
pub struct NoopXiaozhiActivationChallengeRegistry;

impl XiaozhiActivationChallengeRegistry for NoopXiaozhiActivationChallengeRegistry {
    fn register_challenge(&self, _request: &HttpRequest, _challenge: &str, _timeout_ms: u32) {}

    fn consume_challenge(&self, _request: &HttpRequest, _challenge: &str) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct DefaultXiaozhiActivationVerifier {
    challenge_registry: Option<Arc<dyn XiaozhiActivationChallengeRegistry>>,
}

#[derive(Debug, Clone)]
pub struct FileBackedXiaozhiActivationChallengeRegistry {
    path: PathBuf,
    lock_path: PathBuf,
    entries: Arc<Mutex<HashMap<ActivationChallengeKey, ActivationChallengeEntry>>>,
    lock_wait: Duration,
    lock_poll: Duration,
    lock_stale: Duration,
}

#[derive(Debug, Clone)]
pub struct SqliteXiaozhiActivationChallengeRegistry {
    path: PathBuf,
    connection: Arc<Mutex<rusqlite::Connection>>,
}

#[derive(Debug, Clone)]
pub struct RedisXiaozhiActivationChallengeRegistry {
    redis_url: String,
    key_prefix: String,
}

impl FileBackedXiaozhiActivationChallengeRegistry {
    pub fn new(path: PathBuf) -> Self {
        let lock_wait = Duration::from_millis(env_u64(
            ENV_XIAOZHI_ACTIVATION_REGISTRY_LOCK_WAIT_MILLIS,
            DEFAULT_ACTIVATION_REGISTRY_LOCK_WAIT_MILLIS,
        ));
        let lock_poll = Duration::from_millis(
            env_u64(
                ENV_XIAOZHI_ACTIVATION_REGISTRY_LOCK_POLL_MILLIS,
                DEFAULT_ACTIVATION_REGISTRY_LOCK_POLL_MILLIS,
            )
            .max(1),
        );
        let lock_stale = Duration::from_millis(env_u64(
            ENV_XIAOZHI_ACTIVATION_REGISTRY_LOCK_STALE_MILLIS,
            DEFAULT_ACTIVATION_REGISTRY_LOCK_STALE_MILLIS,
        ));
        Self::with_locking(path, lock_wait, lock_poll, lock_stale)
    }

    pub fn with_locking(
        path: PathBuf,
        lock_wait: Duration,
        lock_poll: Duration,
        lock_stale: Duration,
    ) -> Self {
        let entries = load_activation_registry_entries(&path);
        Self {
            lock_path: activation_registry_lock_path(&path),
            path,
            entries: Arc::new(Mutex::new(entries)),
            lock_wait,
            lock_poll,
            lock_stale,
        }
    }

    fn with_locked_disk_entries<T>(
        &self,
        mutate: impl FnOnce(&mut HashMap<ActivationChallengeKey, ActivationChallengeEntry>) -> T,
    ) -> io::Result<T> {
        with_activation_registry_file_lock(
            &self.lock_path,
            self.lock_wait,
            self.lock_poll,
            self.lock_stale,
            || {
                let mut entries = load_activation_registry_entries(&self.path);
                let output = mutate(&mut entries);
                let records = activation_registry_records(&entries);
                write_activation_registry_records(&self.path, &records)?;
                let mut memory_entries = self
                    .entries
                    .lock()
                    .expect("xiaozhi activation challenge registry lock");
                *memory_entries = entries;
                Ok(output)
            },
        )
    }
}

impl SqliteXiaozhiActivationChallengeRegistry {
    pub fn new(path: PathBuf) -> Self {
        let connection = sqlite_connection_for_path(&path);
        let registry = Self { path, connection };
        if let Err(error) = registry.ensure_schema() {
            eprintln!(
                "sdkwork-aiot-gateway activation_registry_sqlite_schema_error path={} error={error}",
                registry.path.display()
            );
        }
        registry
    }

    fn ensure_schema(&self) -> rusqlite::Result<()> {
        let conn = self
            .connection
            .lock()
            .expect("xiaozhi sqlite activation challenge registry lock");
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA busy_timeout=3000;
CREATE TABLE IF NOT EXISTS xiaozhi_activation_challenge_registry(
    device_id TEXT NOT NULL,
    client_id TEXT NOT NULL,
    challenge TEXT NOT NULL,
    expires_at_millis INTEGER NOT NULL,
    PRIMARY KEY(device_id, client_id, challenge)
);",
        )
    }

    fn key_for_request(&self, request: &HttpRequest, challenge: &str) -> ActivationChallengeKey {
        activation_challenge_key(request, challenge)
    }

    fn current_millis(&self) -> i64 {
        current_unix_time_millis()
    }
}

impl RedisXiaozhiActivationChallengeRegistry {
    pub fn new(redis_url: String, key_prefix: Option<String>) -> Self {
        let key_prefix = key_prefix
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "sdkwork:aiot:xiaozhi:activation".to_string());
        Self {
            redis_url,
            key_prefix,
        }
    }

    fn key_for_request(&self, request: &HttpRequest, challenge: &str) -> String {
        let key = activation_challenge_key(request, challenge);
        format!(
            "{}:{}:{}:{}",
            self.key_prefix,
            encode_registry_hex(&key.device_id),
            encode_registry_hex(&key.client_id),
            encode_registry_hex(&key.challenge)
        )
    }

    fn open_connection(&self) -> redis::RedisResult<redis::Connection> {
        let client = redis::Client::open(self.redis_url.as_str())?;
        client.get_connection()
    }
}

impl XiaozhiActivationChallengeRegistry for FileBackedXiaozhiActivationChallengeRegistry {
    fn register_challenge(&self, request: &HttpRequest, challenge: &str, timeout_ms: u32) {
        activation_registry_mark_backend(ACTIVATION_REGISTRY_BACKEND_FILE);
        ACTIVATION_REGISTRY_STATS
            .register_total
            .fetch_add(1, Ordering::Relaxed);
        let result = self.with_locked_disk_entries(|entries| {
            register_challenge_in_entries(entries, request, challenge, timeout_ms)
        });
        match result {
            Ok(pruned) => activation_registry_add_pruned(pruned),
            Err(error) => {
                eprintln!(
                    "sdkwork-aiot-gateway activation_registry_register_error path={} error={error}",
                    self.path.display()
                );
            }
        }
    }

    fn consume_challenge(&self, request: &HttpRequest, challenge: &str) -> bool {
        activation_registry_mark_backend(ACTIVATION_REGISTRY_BACKEND_FILE);
        ACTIVATION_REGISTRY_STATS
            .consume_total
            .fetch_add(1, Ordering::Relaxed);
        match self.with_locked_disk_entries(|entries| {
            consume_challenge_in_entries(entries, request, challenge)
        }) {
            Ok((consumed, pruned)) => {
                activation_registry_add_pruned(pruned);
                activation_registry_record_consume_outcome(consumed);
                consumed
            }
            Err(error) => {
                eprintln!(
                    "sdkwork-aiot-gateway activation_registry_consume_error path={} error={error}",
                    self.path.display()
                );
                activation_registry_record_consume_outcome(false);
                false
            }
        }
    }
}

impl XiaozhiActivationChallengeRegistry for SqliteXiaozhiActivationChallengeRegistry {
    fn register_challenge(&self, request: &HttpRequest, challenge: &str, timeout_ms: u32) {
        activation_registry_mark_backend(ACTIVATION_REGISTRY_BACKEND_SQLITE);
        ACTIVATION_REGISTRY_STATS
            .register_total
            .fetch_add(1, Ordering::Relaxed);
        let key = self.key_for_request(request, challenge);
        let now = self.current_millis();
        let expires_at_millis = now.saturating_add(i64::from(timeout_ms));

        let mut conn = self
            .connection
            .lock()
            .expect("xiaozhi sqlite activation challenge registry lock");

        let tx = match conn.transaction() {
            Ok(tx) => tx,
            Err(error) => {
                eprintln!(
                    "sdkwork-aiot-gateway activation_registry_sqlite_register_tx_error path={} error={error}",
                    self.path.display()
                );
                return;
            }
        };

        let pruned = match tx.execute(
            "DELETE FROM xiaozhi_activation_challenge_registry WHERE expires_at_millis <= ?1",
            [now],
        ) {
            Ok(count) => count as u64,
            Err(error) => {
                eprintln!(
                    "sdkwork-aiot-gateway activation_registry_sqlite_prune_error path={} error={error}",
                    self.path.display()
                );
                let _ = tx.rollback();
                return;
            }
        };

        if let Err(error) = tx.execute(
            "INSERT INTO xiaozhi_activation_challenge_registry(device_id, client_id, challenge, expires_at_millis)
VALUES(?1, ?2, ?3, ?4)
ON CONFLICT(device_id, client_id, challenge)
DO UPDATE SET expires_at_millis=excluded.expires_at_millis",
            rusqlite::params![
                key.device_id,
                key.client_id,
                key.challenge,
                expires_at_millis
            ],
        ) {
            eprintln!(
                "sdkwork-aiot-gateway activation_registry_sqlite_register_error path={} error={error}",
                self.path.display()
            );
            let _ = tx.rollback();
            return;
        }

        if let Err(error) = tx.commit() {
            eprintln!(
                "sdkwork-aiot-gateway activation_registry_sqlite_register_commit_error path={} error={error}",
                self.path.display()
            );
            return;
        }
        activation_registry_add_pruned(pruned);
    }

    fn consume_challenge(&self, request: &HttpRequest, challenge: &str) -> bool {
        activation_registry_mark_backend(ACTIVATION_REGISTRY_BACKEND_SQLITE);
        ACTIVATION_REGISTRY_STATS
            .consume_total
            .fetch_add(1, Ordering::Relaxed);
        let key = self.key_for_request(request, challenge);
        let now = self.current_millis();

        let mut conn = self
            .connection
            .lock()
            .expect("xiaozhi sqlite activation challenge registry lock");

        let tx = match conn.transaction() {
            Ok(tx) => tx,
            Err(error) => {
                eprintln!(
                    "sdkwork-aiot-gateway activation_registry_sqlite_consume_tx_error path={} error={error}",
                    self.path.display()
                );
                activation_registry_record_consume_outcome(false);
                return false;
            }
        };

        let pruned = match tx.execute(
            "DELETE FROM xiaozhi_activation_challenge_registry WHERE expires_at_millis <= ?1",
            [now],
        ) {
            Ok(count) => count as u64,
            Err(error) => {
                eprintln!(
                    "sdkwork-aiot-gateway activation_registry_sqlite_prune_error path={} error={error}",
                    self.path.display()
                );
                let _ = tx.rollback();
                activation_registry_record_consume_outcome(false);
                return false;
            }
        };

        let deleted = match tx.execute(
            "DELETE FROM xiaozhi_activation_challenge_registry
WHERE device_id=?1 AND client_id=?2 AND challenge=?3 AND expires_at_millis > ?4",
            rusqlite::params![key.device_id, key.client_id, key.challenge, now],
        ) {
            Ok(count) => count > 0,
            Err(error) => {
                eprintln!(
                    "sdkwork-aiot-gateway activation_registry_sqlite_consume_error path={} error={error}",
                    self.path.display()
                );
                let _ = tx.rollback();
                activation_registry_record_consume_outcome(false);
                return false;
            }
        };

        if let Err(error) = tx.commit() {
            eprintln!(
                "sdkwork-aiot-gateway activation_registry_sqlite_consume_commit_error path={} error={error}",
                self.path.display()
            );
            activation_registry_record_consume_outcome(false);
            return false;
        }

        activation_registry_add_pruned(pruned);
        activation_registry_record_consume_outcome(deleted);
        deleted
    }
}

impl XiaozhiActivationChallengeRegistry for RedisXiaozhiActivationChallengeRegistry {
    fn register_challenge(&self, request: &HttpRequest, challenge: &str, timeout_ms: u32) {
        activation_registry_mark_backend(ACTIVATION_REGISTRY_BACKEND_REDIS);
        ACTIVATION_REGISTRY_STATS
            .register_total
            .fetch_add(1, Ordering::Relaxed);
        let key = self.key_for_request(request, challenge);
        let ttl_millis = u64::from(timeout_ms).max(1);
        let mut conn = match self.open_connection() {
            Ok(conn) => conn,
            Err(error) => {
                eprintln!(
                    "sdkwork-aiot-gateway activation_registry_redis_connect_error url={} error={error}",
                    self.redis_url
                );
                return;
            }
        };
        let result = redis::cmd("PSETEX")
            .arg(&key)
            .arg(ttl_millis)
            .arg("1")
            .query::<String>(&mut conn);
        if let Err(error) = result {
            eprintln!(
                "sdkwork-aiot-gateway activation_registry_redis_register_error url={} key={} error={error}",
                self.redis_url, key
            );
        }
    }

    fn consume_challenge(&self, request: &HttpRequest, challenge: &str) -> bool {
        activation_registry_mark_backend(ACTIVATION_REGISTRY_BACKEND_REDIS);
        ACTIVATION_REGISTRY_STATS
            .consume_total
            .fetch_add(1, Ordering::Relaxed);
        let key = self.key_for_request(request, challenge);
        let mut conn = match self.open_connection() {
            Ok(conn) => conn,
            Err(error) => {
                eprintln!(
                    "sdkwork-aiot-gateway activation_registry_redis_connect_error url={} error={error}",
                    self.redis_url
                );
                activation_registry_record_consume_outcome(false);
                return false;
            }
        };

        let script = redis::Script::new(
            r#"if redis.call("EXISTS", KEYS[1]) == 1 then
return redis.call("DEL", KEYS[1])
else
return 0
end"#,
        );
        let deleted = match script.key(&key).invoke::<u64>(&mut conn) {
            Ok(value) => value > 0,
            Err(error) => {
                eprintln!(
                    "sdkwork-aiot-gateway activation_registry_redis_consume_error url={} key={} error={error}",
                    self.redis_url, key
                );
                activation_registry_record_consume_outcome(false);
                return false;
            }
        };
        activation_registry_record_consume_outcome(deleted);
        deleted
    }
}

impl DefaultXiaozhiActivationVerifier {
    pub fn stateless() -> Self {
        Self {
            challenge_registry: None,
        }
    }

    pub fn with_challenge_registry(
        challenge_registry: Arc<dyn XiaozhiActivationChallengeRegistry>,
    ) -> Self {
        Self {
            challenge_registry: Some(challenge_registry),
        }
    }
}

impl XiaozhiActivationVerifier for DefaultXiaozhiActivationVerifier {
    fn is_accepted(&self, request: &HttpRequest) -> bool {
        activation_request_accepted(request, self.challenge_registry.as_deref())
    }
}

pub trait XiaozhiOtaProfileProvider: Send + Sync {
    fn enrich(&self, request: &HttpRequest, metadata: XiaozhiOtaMetadata) -> XiaozhiOtaMetadata;
}

#[derive(Debug, Clone)]
pub struct DefaultXiaozhiOtaProfileProvider;

impl XiaozhiOtaProfileProvider for DefaultXiaozhiOtaProfileProvider {
    fn enrich(
        &self,
        _request: &HttpRequest,
        mut metadata: XiaozhiOtaMetadata,
    ) -> XiaozhiOtaMetadata {
        if let Some((endpoint, client_id, username, password, publish_topic, subscribe_topic)) =
            mqtt_ota_from_env()
        {
            metadata = metadata.with_mqtt(
                endpoint,
                client_id,
                username,
                password,
                publish_topic,
                subscribe_topic,
                env_u32(ENV_XIAOZHI_MQTT_KEEPALIVE, DEFAULT_MQTT_KEEPALIVE_SECONDS),
            );
        }

        if let Some((server, port, key_hex, nonce_hex)) = mqtt_udp_profile_from_env() {
            metadata = metadata.with_mqtt_udp(server, port, key_hex, nonce_hex);
        }

        if let Some((firmware_version, firmware_url, force)) = firmware_ota_from_env() {
            metadata = metadata.with_firmware(firmware_version, firmware_url, force);
        }

        if let Some((message, code, challenge, timeout_ms)) = activation_profile_from_env() {
            metadata = if let Some(challenge) = challenge {
                metadata.with_activation_challenge(message, challenge, timeout_ms)
            } else if let Some(code) = code {
                metadata.with_activation_code(message, code, timeout_ms)
            } else {
                metadata
            };
        }

        metadata
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketSessionReply {
    Text(String),
    Binary(Vec<u8>),
    Pong(Vec<u8>),
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MqttSessionReply {
    pub outbound_json: Vec<String>,
    pub close_audio_channel: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XiaozhiSimulatorMcpToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema_json: String,
    user_only: bool,
    simulated_result_text: Option<String>,
}

impl XiaozhiSimulatorMcpToolSpec {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema_json: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema_json: input_schema_json.into(),
            user_only: false,
            simulated_result_text: None,
        }
    }

    pub fn with_user_only(mut self, user_only: bool) -> Self {
        self.user_only = user_only;
        self
    }

    pub fn user_only(&self) -> bool {
        self.user_only
    }

    pub fn with_simulated_result_text(mut self, result_text: impl Into<String>) -> Self {
        let result_text = result_text.into();
        if result_text.trim().is_empty() {
            self.simulated_result_text = None;
        } else {
            self.simulated_result_text = Some(result_text);
        }
        self
    }

    pub fn simulated_result_text(&self) -> Option<&str> {
        self.simulated_result_text.as_deref()
    }
}

pub trait XiaozhiSimulatorMcpToolProvider: Send + Sync {
    fn tools(&self) -> Vec<XiaozhiSimulatorMcpToolSpec>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XiaozhiMcpInvocationContext {
    pub transport: String,
    pub session_id: String,
    pub device_id: Option<String>,
    pub client_id: Option<String>,
}

impl XiaozhiMcpInvocationContext {
    pub fn new(transport: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            transport: transport.into(),
            session_id: session_id.into(),
            device_id: None,
            client_id: None,
        }
    }

    pub fn with_device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }

    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }
}

pub trait XiaozhiSimulatorMcpToolInvoker: Send + Sync {
    fn invoke(
        &self,
        context: &XiaozhiMcpInvocationContext,
        tool: &XiaozhiSimulatorMcpToolSpec,
        tool_arguments_json: Option<&str>,
    ) -> Result<String, String>;
}

pub trait XiaozhiSimulatorMcpToolPolicy: Send + Sync {
    fn allow(
        &self,
        context: &XiaozhiMcpInvocationContext,
        tool: &XiaozhiSimulatorMcpToolSpec,
        tool_arguments_json: Option<&str>,
    ) -> Result<(), String>;

    fn evaluate(
        &self,
        context: &XiaozhiMcpInvocationContext,
        tool: &XiaozhiSimulatorMcpToolSpec,
        tool_arguments_json: Option<&str>,
    ) -> XiaozhiMcpPolicyEvaluation {
        match self.allow(context, tool, tool_arguments_json) {
            Ok(()) => XiaozhiMcpPolicyEvaluation::allow(None),
            Err(error_message) => XiaozhiMcpPolicyEvaluation::deny(error_message, None),
        }
    }

    fn stats_snapshot(&self) -> Option<XiaozhiMcpPolicyStatsSnapshot> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XiaozhiMcpPolicyDecision {
    Allow,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XiaozhiMcpPolicyEvaluation {
    pub decision: XiaozhiMcpPolicyDecision,
    pub matched_rule_index: Option<usize>,
    pub error_message: Option<String>,
}

impl XiaozhiMcpPolicyEvaluation {
    pub fn allow(matched_rule_index: Option<usize>) -> Self {
        Self {
            decision: XiaozhiMcpPolicyDecision::Allow,
            matched_rule_index,
            error_message: None,
        }
    }

    pub fn deny(error_message: impl Into<String>, matched_rule_index: Option<usize>) -> Self {
        Self {
            decision: XiaozhiMcpPolicyDecision::Deny,
            matched_rule_index,
            error_message: Some(error_message.into()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AllowAllXiaozhiSimulatorMcpToolPolicy;

impl XiaozhiSimulatorMcpToolPolicy for AllowAllXiaozhiSimulatorMcpToolPolicy {
    fn allow(
        &self,
        _context: &XiaozhiMcpInvocationContext,
        _tool: &XiaozhiSimulatorMcpToolSpec,
        _tool_arguments_json: Option<&str>,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum McpPolicyDecision {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum McpPolicyNumericOperator {
    Gte,
    Lte,
    Gt,
    Lt,
    Eq,
    Ne,
}

#[derive(Debug, Clone, PartialEq)]
struct McpPolicyNumericArgumentPredicate {
    field: String,
    operator: McpPolicyNumericOperator,
    expected: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum McpPolicyStringOperator {
    Eq,
    Ne,
    Prefix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct McpPolicyStringArgumentPredicate {
    field: String,
    operator: McpPolicyStringOperator,
    expected: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum McpPolicyBooleanOperator {
    Eq,
    Ne,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct McpPolicyBooleanArgumentPredicate {
    field: String,
    operator: McpPolicyBooleanOperator,
    expected: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct McpPolicyRulePattern {
    tool: Option<String>,
    transport: Option<String>,
    device_prefix: Option<String>,
    client_prefix: Option<String>,
    numeric_arg_predicates: Vec<McpPolicyNumericArgumentPredicate>,
    string_arg_predicates: Vec<McpPolicyStringArgumentPredicate>,
    boolean_arg_predicates: Vec<McpPolicyBooleanArgumentPredicate>,
}

#[derive(Debug, Clone, PartialEq)]
struct McpPolicyRule {
    decision: McpPolicyDecision,
    pattern: McpPolicyRulePattern,
}

#[derive(Debug, Default)]
struct RuleBasedMcpPolicyStats {
    allow_by_rule_matches: AtomicU64,
    allow_no_rule_matches: AtomicU64,
    deny_by_rule_matches: AtomicU64,
}

impl RuleBasedMcpPolicyStats {
    fn on_allow_by_rule(&self) {
        self.allow_by_rule_matches.fetch_add(1, Ordering::Relaxed);
    }

    fn on_allow_no_rule(&self) {
        self.allow_no_rule_matches.fetch_add(1, Ordering::Relaxed);
    }

    fn on_deny_by_rule(&self) {
        self.deny_by_rule_matches.fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> XiaozhiMcpPolicyStatsSnapshot {
        XiaozhiMcpPolicyStatsSnapshot {
            allow_by_rule_matches: self.allow_by_rule_matches.load(Ordering::Relaxed),
            allow_no_rule_matches: self.allow_no_rule_matches.load(Ordering::Relaxed),
            deny_by_rule_matches: self.deny_by_rule_matches.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XiaozhiMcpPolicyStatsSnapshot {
    pub allow_by_rule_matches: u64,
    pub allow_no_rule_matches: u64,
    pub deny_by_rule_matches: u64,
}

#[derive(Debug, Clone, Default)]
pub struct RuleBasedXiaozhiSimulatorMcpToolPolicy {
    rules: Vec<McpPolicyRule>,
    stats: Arc<RuleBasedMcpPolicyStats>,
}

impl RuleBasedXiaozhiSimulatorMcpToolPolicy {
    #[cfg(test)]
    fn from_rules(rules: Vec<McpPolicyRule>) -> Self {
        Self {
            rules,
            stats: Arc::new(RuleBasedMcpPolicyStats::default()),
        }
    }

    pub fn from_env() -> Self {
        let rules = env_string(ENV_XIAOZHI_MCP_POLICY_RULES)
            .map(|raw| parse_mcp_policy_rules(raw.as_str()))
            .unwrap_or_default();
        Self {
            rules,
            stats: Arc::new(RuleBasedMcpPolicyStats::default()),
        }
    }

    pub fn stats_snapshot(&self) -> XiaozhiMcpPolicyStatsSnapshot {
        self.stats.snapshot()
    }

    fn evaluate_rule(
        &self,
        context: &XiaozhiMcpInvocationContext,
        tool: &XiaozhiSimulatorMcpToolSpec,
        tool_arguments_json: Option<&str>,
    ) -> XiaozhiMcpPolicyEvaluation {
        let matched_rule =
            self.rules.iter().enumerate().find(|(_, rule)| {
                mcp_policy_rule_matches(rule, context, tool, tool_arguments_json)
            });

        let Some((matched_rule_index, matched_rule)) = matched_rule else {
            self.stats.on_allow_no_rule();
            return XiaozhiMcpPolicyEvaluation::allow(None);
        };

        match matched_rule.decision {
            McpPolicyDecision::Allow => {
                self.stats.on_allow_by_rule();
                XiaozhiMcpPolicyEvaluation::allow(Some(matched_rule_index))
            }
            McpPolicyDecision::Deny => {
                self.stats.on_deny_by_rule();
                XiaozhiMcpPolicyEvaluation::deny(
                    format!("Tool not allowed by policy: {}", tool.name),
                    Some(matched_rule_index),
                )
            }
        }
    }
}

impl XiaozhiSimulatorMcpToolPolicy for RuleBasedXiaozhiSimulatorMcpToolPolicy {
    fn allow(
        &self,
        context: &XiaozhiMcpInvocationContext,
        tool: &XiaozhiSimulatorMcpToolSpec,
        tool_arguments_json: Option<&str>,
    ) -> Result<(), String> {
        let evaluation = self.evaluate_rule(context, tool, tool_arguments_json);
        match evaluation.decision {
            XiaozhiMcpPolicyDecision::Allow => Ok(()),
            XiaozhiMcpPolicyDecision::Deny => Err(evaluation
                .error_message
                .unwrap_or_else(|| "Tool not allowed by policy".to_string())),
        }
    }

    fn evaluate(
        &self,
        context: &XiaozhiMcpInvocationContext,
        tool: &XiaozhiSimulatorMcpToolSpec,
        tool_arguments_json: Option<&str>,
    ) -> XiaozhiMcpPolicyEvaluation {
        self.evaluate_rule(context, tool, tool_arguments_json)
    }

    fn stats_snapshot(&self) -> Option<XiaozhiMcpPolicyStatsSnapshot> {
        Some(RuleBasedXiaozhiSimulatorMcpToolPolicy::stats_snapshot(self))
    }
}

#[derive(Debug, Clone, Default)]
pub struct DefaultXiaozhiSimulatorMcpToolInvoker;

impl XiaozhiSimulatorMcpToolInvoker for DefaultXiaozhiSimulatorMcpToolInvoker {
    fn invoke(
        &self,
        _context: &XiaozhiMcpInvocationContext,
        tool: &XiaozhiSimulatorMcpToolSpec,
        _tool_arguments_json: Option<&str>,
    ) -> Result<String, String> {
        Ok(tool
            .simulated_result_text()
            .unwrap_or("accepted by SDKWork simulator")
            .to_string())
    }
}

#[derive(Debug, Clone)]
pub struct DefaultXiaozhiSimulatorMcpToolProvider {
    tools: Vec<XiaozhiSimulatorMcpToolSpec>,
}

impl DefaultXiaozhiSimulatorMcpToolProvider {
    pub fn from_path(path: &Path) -> Option<Self> {
        let tools = read_simulator_mcp_tools_file(path)?;
        if tools.is_empty() {
            None
        } else {
            Some(Self { tools })
        }
    }

    pub fn from_tools(tools: Vec<XiaozhiSimulatorMcpToolSpec>) -> Self {
        Self { tools }
    }

    pub fn from_env() -> Self {
        if let Some(path) = env_string(ENV_XIAOZHI_SIMULATOR_MCP_TOOLS_PATH) {
            let path = PathBuf::from(path);
            if let Some(provider) = Self::from_path(&path) {
                return provider;
            }
        }
        Self::from_tools(built_in_simulator_mcp_tools())
    }
}

impl Default for DefaultXiaozhiSimulatorMcpToolProvider {
    fn default() -> Self {
        Self::from_env()
    }
}

impl XiaozhiSimulatorMcpToolProvider for DefaultXiaozhiSimulatorMcpToolProvider {
    fn tools(&self) -> Vec<XiaozhiSimulatorMcpToolSpec> {
        self.tools.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XiaozhiMqttUdpSession {
    pub device_id: String,
    pub client_id: String,
    pub session_id: String,
    pub udp_server: String,
    pub udp_port: u16,
    pub udp_key_hex: String,
    pub udp_nonce_hex: String,
    pub remote_sequence: u32,
}

impl XiaozhiMqttUdpSession {
    pub fn udp_codec(&self) -> Result<XiaozhiUdpAudioCodec, TransportError> {
        XiaozhiUdpAudioCodec::new(&self.udp_key_hex, &self.udp_nonce_hex)
            .map_err(|error| TransportError::new(error.code))
    }
}

#[derive(Clone)]
pub struct XiaozhiSessionOptions {
    mcp_tool_provider: Arc<dyn XiaozhiSimulatorMcpToolProvider>,
    mcp_tool_invoker: Arc<dyn XiaozhiSimulatorMcpToolInvoker>,
    mcp_tool_policy: Arc<dyn XiaozhiSimulatorMcpToolPolicy>,
}

impl XiaozhiSessionOptions {
    pub fn from_env() -> Self {
        Self {
            mcp_tool_provider: Arc::new(DefaultXiaozhiSimulatorMcpToolProvider::from_env()),
            mcp_tool_invoker: Arc::new(DefaultXiaozhiSimulatorMcpToolInvoker),
            mcp_tool_policy: Arc::new(RuleBasedXiaozhiSimulatorMcpToolPolicy::from_env()),
        }
    }

    pub fn from_mcp_tool_provider(
        mcp_tool_provider: Arc<dyn XiaozhiSimulatorMcpToolProvider>,
    ) -> Self {
        Self {
            mcp_tool_provider,
            mcp_tool_invoker: Arc::new(DefaultXiaozhiSimulatorMcpToolInvoker),
            mcp_tool_policy: Arc::new(AllowAllXiaozhiSimulatorMcpToolPolicy),
        }
    }

    pub fn from_mcp_tool_provider_and_invoker(
        mcp_tool_provider: Arc<dyn XiaozhiSimulatorMcpToolProvider>,
        mcp_tool_invoker: Arc<dyn XiaozhiSimulatorMcpToolInvoker>,
    ) -> Self {
        Self {
            mcp_tool_provider,
            mcp_tool_invoker,
            mcp_tool_policy: Arc::new(AllowAllXiaozhiSimulatorMcpToolPolicy),
        }
    }

    pub fn from_mcp_tool_provider_invoker_and_policy(
        mcp_tool_provider: Arc<dyn XiaozhiSimulatorMcpToolProvider>,
        mcp_tool_invoker: Arc<dyn XiaozhiSimulatorMcpToolInvoker>,
        mcp_tool_policy: Arc<dyn XiaozhiSimulatorMcpToolPolicy>,
    ) -> Self {
        Self {
            mcp_tool_provider,
            mcp_tool_invoker,
            mcp_tool_policy,
        }
    }

    pub fn mcp_tool_provider(&self) -> Arc<dyn XiaozhiSimulatorMcpToolProvider> {
        Arc::clone(&self.mcp_tool_provider)
    }

    pub fn mcp_tool_invoker(&self) -> Arc<dyn XiaozhiSimulatorMcpToolInvoker> {
        Arc::clone(&self.mcp_tool_invoker)
    }

    pub fn mcp_tool_policy(&self) -> Arc<dyn XiaozhiSimulatorMcpToolPolicy> {
        Arc::clone(&self.mcp_tool_policy)
    }
}

pub fn standard_gateway_server() -> Result<TransportServer, TransportError> {
    let (server, _session_options) = standard_gateway_server_and_session_options()?;
    Ok(server)
}

pub fn standard_gateway_server_and_session_options(
) -> Result<(TransportServer, XiaozhiSessionOptions), TransportError> {
    let challenge_registry: Arc<dyn XiaozhiActivationChallengeRegistry> =
        activation_challenge_registry_from_env();
    let mcp_tool_provider: Arc<dyn XiaozhiSimulatorMcpToolProvider> =
        Arc::new(DefaultXiaozhiSimulatorMcpToolProvider::from_env());
    standard_gateway_server_and_session_options_with_plugins_activation_registry_and_mcp_tools(
        Arc::new(DefaultXiaozhiOtaProfileProvider),
        Arc::new(DefaultXiaozhiActivationVerifier::with_challenge_registry(
            Arc::clone(&challenge_registry),
        )),
        challenge_registry,
        mcp_tool_provider,
    )
}

pub fn standard_gateway_server_with_plugins(
    ota_provider: Arc<dyn XiaozhiOtaProfileProvider>,
    activation_verifier: Arc<dyn XiaozhiActivationVerifier>,
) -> Result<TransportServer, TransportError> {
    let (server, _session_options) =
        standard_gateway_server_and_session_options_with_plugins_activation_registry_and_mcp_tools(
            ota_provider,
            activation_verifier,
            Arc::new(InMemoryXiaozhiActivationChallengeRegistry::new()),
            Arc::new(DefaultXiaozhiSimulatorMcpToolProvider::from_env()),
        )?;
    Ok(server)
}

pub fn standard_gateway_server_with_plugins_and_activation_registry(
    ota_provider: Arc<dyn XiaozhiOtaProfileProvider>,
    activation_verifier: Arc<dyn XiaozhiActivationVerifier>,
    challenge_registry: Arc<dyn XiaozhiActivationChallengeRegistry>,
) -> Result<TransportServer, TransportError> {
    let (server, _session_options) =
        standard_gateway_server_and_session_options_with_plugins_activation_registry_and_mcp_tools(
            ota_provider,
            activation_verifier,
            challenge_registry,
            Arc::new(DefaultXiaozhiSimulatorMcpToolProvider::from_env()),
        )?;
    Ok(server)
}

pub fn standard_gateway_server_with_plugins_activation_registry_and_mcp_tools(
    ota_provider: Arc<dyn XiaozhiOtaProfileProvider>,
    activation_verifier: Arc<dyn XiaozhiActivationVerifier>,
    challenge_registry: Arc<dyn XiaozhiActivationChallengeRegistry>,
    mcp_tool_provider: Arc<dyn XiaozhiSimulatorMcpToolProvider>,
) -> Result<TransportServer, TransportError> {
    let (server, _session_options) =
        standard_gateway_server_and_session_options_with_plugins_activation_registry_and_mcp_tools(
            ota_provider,
            activation_verifier,
            challenge_registry,
            mcp_tool_provider,
        )?;
    Ok(server)
}

pub fn standard_gateway_server_and_session_options_with_plugins_activation_registry_and_mcp_tools(
    ota_provider: Arc<dyn XiaozhiOtaProfileProvider>,
    activation_verifier: Arc<dyn XiaozhiActivationVerifier>,
    challenge_registry: Arc<dyn XiaozhiActivationChallengeRegistry>,
    mcp_tool_provider: Arc<dyn XiaozhiSimulatorMcpToolProvider>,
) -> Result<(TransportServer, XiaozhiSessionOptions), TransportError> {
    let server = build_standard_gateway_transport_server(
        ota_provider,
        activation_verifier,
        challenge_registry,
    )?;
    let session_options = XiaozhiSessionOptions::from_mcp_tool_provider_invoker_and_policy(
        mcp_tool_provider,
        Arc::new(DefaultXiaozhiSimulatorMcpToolInvoker),
        Arc::new(RuleBasedXiaozhiSimulatorMcpToolPolicy::from_env()),
    );
    Ok((server, session_options))
}

fn build_standard_gateway_transport_server(
    ota_provider: Arc<dyn XiaozhiOtaProfileProvider>,
    activation_verifier: Arc<dyn XiaozhiActivationVerifier>,
    challenge_registry: Arc<dyn XiaozhiActivationChallengeRegistry>,
) -> Result<TransportServer, TransportError> {
    let activation_verifier_alias = Arc::clone(&activation_verifier);
    Ok(TransportServer::standard_standalone()?
        .with_http_compatibility_route(XIAOZHI_OTA_PATH, move |request| {
            xiaozhi_ota_http_handler_with_provider_and_registry(
                request,
                ota_provider.as_ref(),
                challenge_registry.as_ref(),
            )
        })
        .with_http_compatibility_route(XIAOZHI_ACTIVATE_PATH, move |request| {
            xiaozhi_activation_http_handler_with_verifier(request, activation_verifier.as_ref())
        })
        .with_http_compatibility_route(XIAOZHI_OTA_ACTIVATE_PATH, move |request| {
            xiaozhi_activation_http_handler_with_verifier(
                request,
                activation_verifier_alias.as_ref(),
            )
        })
        .with_http_compatibility_route(XIAOZHI_SIMULATOR_PATH, xiaozhi_simulator_http_handler))
}

pub fn xiaozhi_ota_http_handler(request: &HttpRequest) -> HttpResponse {
    xiaozhi_ota_http_handler_with_provider(request, &DefaultXiaozhiOtaProfileProvider)
}

pub fn xiaozhi_ota_http_handler_with_provider(
    request: &HttpRequest,
    provider: &dyn XiaozhiOtaProfileProvider,
) -> HttpResponse {
    xiaozhi_ota_http_handler_with_provider_and_registry(
        request,
        provider,
        &NoopXiaozhiActivationChallengeRegistry,
    )
}

pub fn xiaozhi_ota_http_handler_with_provider_and_registry(
    request: &HttpRequest,
    provider: &dyn XiaozhiOtaProfileProvider,
    challenge_registry: &dyn XiaozhiActivationChallengeRegistry,
) -> HttpResponse {
    if request.method != "POST" && request.method != "GET" {
        return problem_response(HttpStatus::BadRequest, "gateway.xiaozhi.ota.method");
    }

    let host = request.header("host").unwrap_or("localhost");
    let ws_scheme = websocket_scheme(request);
    let token = request
        .header("authorization")
        .map(str::to_string)
        .or_else(|| env_string(ENV_XIAOZHI_DEVICE_TOKEN))
        .unwrap_or_else(|| DEFAULT_DEVICE_TOKEN.to_string());
    let version = request
        .header("protocol-version")
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(3);

    let metadata = XiaozhiOtaMetadata::new()
        .with_websocket(
            format!("{ws_scheme}://{host}{XIAOZHI_WS_PATH}"),
            token,
            version,
        )
        .with_server_time(
            current_unix_time_millis(),
            env_i32(
                ENV_XIAOZHI_SERVER_TIMEZONE_OFFSET_MINUTES,
                DEFAULT_SERVER_TIMEZONE_OFFSET_MINUTES,
            ),
        );
    let metadata = provider.enrich(request, metadata);
    register_activation_challenge_if_present(challenge_registry, request, &metadata);

    HttpResponse::new(HttpStatus::Ok)
        .with_header("content-type", "application/json")
        .with_body(xiaozhi_ota_response(metadata))
}

pub fn xiaozhi_activation_http_handler(request: &HttpRequest) -> HttpResponse {
    xiaozhi_activation_http_handler_with_verifier(
        request,
        &DefaultXiaozhiActivationVerifier::stateless(),
    )
}

pub fn xiaozhi_activation_http_handler_with_verifier(
    request: &HttpRequest,
    verifier: &dyn XiaozhiActivationVerifier,
) -> HttpResponse {
    if request.method != "POST" {
        return problem_response(HttpStatus::BadRequest, "gateway.xiaozhi.activate.method");
    }

    if verifier.is_accepted(request) {
        return HttpResponse::new(HttpStatus::Ok)
            .with_header("content-type", "application/json")
            .with_body(xiaozhi_activation_accepted_response());
    }

    HttpResponse::new(HttpStatus::Accepted)
        .with_header("content-type", "application/json")
        .with_body(xiaozhi_activation_pending_response(
            env_string(ENV_XIAOZHI_ACTIVATION_MESSAGE)
                .as_deref()
                .unwrap_or(DEFAULT_ACTIVATION_MESSAGE),
        ))
}

pub fn xiaozhi_simulator_http_handler(request: &HttpRequest) -> HttpResponse {
    if request.method != "GET" {
        return problem_response(HttpStatus::BadRequest, "gateway.xiaozhi.simulator.method");
    }

    HttpResponse::new(HttpStatus::NotFound)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"code":"gateway.xiaozhi.simulator.ui.migrated","message":"Use the cross-platform simulator binary: cargo run -p sdkwork-aiot-xiaozhi-simulator-ui"}"#,
        )
}

pub fn xiaozhi_websocket_session_reply(
    server: &TransportServer,
    request: &HttpRequest,
    frame: WebSocketFrame,
) -> Result<Vec<WebSocketSessionReply>, TransportError> {
    let options = XiaozhiSessionOptions::from_env();
    xiaozhi_websocket_session_reply_with_options(server, request, frame, &options)
}

pub fn xiaozhi_websocket_session_reply_with_options(
    server: &TransportServer,
    request: &HttpRequest,
    frame: WebSocketFrame,
    options: &XiaozhiSessionOptions,
) -> Result<Vec<WebSocketSessionReply>, TransportError> {
    let mcp_tool_provider = options.mcp_tool_provider();
    let mcp_tool_invoker = options.mcp_tool_invoker();
    let mcp_tool_policy = options.mcp_tool_policy();
    xiaozhi_websocket_session_reply_with_mcp_tool_provider_and_invoker(
        server,
        request,
        frame,
        mcp_tool_provider.as_ref(),
        mcp_tool_invoker.as_ref(),
        mcp_tool_policy.as_ref(),
    )
}

pub fn xiaozhi_websocket_session_reply_with_mcp_tool_provider(
    server: &TransportServer,
    request: &HttpRequest,
    frame: WebSocketFrame,
    mcp_tool_provider: &dyn XiaozhiSimulatorMcpToolProvider,
) -> Result<Vec<WebSocketSessionReply>, TransportError> {
    let mcp_tool_invoker = DefaultXiaozhiSimulatorMcpToolInvoker;
    let mcp_tool_policy = AllowAllXiaozhiSimulatorMcpToolPolicy;
    xiaozhi_websocket_session_reply_with_mcp_tool_provider_and_invoker(
        server,
        request,
        frame,
        mcp_tool_provider,
        &mcp_tool_invoker,
        &mcp_tool_policy,
    )
}

pub fn xiaozhi_websocket_session_reply_with_mcp_tool_provider_and_invoker(
    server: &TransportServer,
    request: &HttpRequest,
    frame: WebSocketFrame,
    mcp_tool_provider: &dyn XiaozhiSimulatorMcpToolProvider,
    mcp_tool_invoker: &dyn XiaozhiSimulatorMcpToolInvoker,
    mcp_tool_policy: &dyn XiaozhiSimulatorMcpToolPolicy,
) -> Result<Vec<WebSocketSessionReply>, TransportError> {
    match frame.opcode {
        WebSocketOpcode::Ping => {
            return Ok(vec![WebSocketSessionReply::Pong(frame.payload)]);
        }
        WebSocketOpcode::Pong => {
            return Ok(Vec::new());
        }
        WebSocketOpcode::Close => {
            return Ok(vec![WebSocketSessionReply::Close]);
        }
        WebSocketOpcode::Text | WebSocketOpcode::Binary => {}
    }

    let codec = xiaozhi_codec_from_request(request);
    let inbound = websocket_frame_to_inbound_frame(frame.clone());
    let result = server
        .runtime
        .handle_inbound_frame_with_codec(XIAOZHI_WS_PATH, &codec, inbound)
        .map_err(TransportError::from_runtime_protocol)?;

    let session_id = result.message.session_id.unwrap_or_else(|| {
        let device_id = result
            .message
            .device_id
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or("device");
        let client_id = result
            .message
            .client_id
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or("client");
        format!("{device_id}-{client_id}")
    });

    let mut replies = Vec::new();
    match result.envelope.semantic_type.as_str() {
        "hello" => {
            replies.push(WebSocketSessionReply::Text(xiaozhi_server_hello_response(
                XiaozhiServerHello::websocket(session_id.clone())
                    .with_audio_params(XiaozhiAudioParams::opus(24_000, 1, 60)),
            )));
            if result
                .envelope
                .extensions
                .get("xiaozhi.feature.mcp")
                .is_some_and(|value| value == "true")
            {
                replies.push(WebSocketSessionReply::Text(format!(
                    r#"{{"session_id":"{}","type":"mcp","payload":{{"jsonrpc":"2.0","method":"initialize","params":{{"capabilities":{{"vision":{{"url":"http://localhost/iot/xiaozhi/vision","token":"simulator-token"}}}}}},"id":1}}}}"#,
                    json_escape(&session_id)
                )));
            }
        }
        "listen" => {
            let state = result
                .envelope
                .extensions
                .get("xiaozhi.listen.state")
                .map(String::as_str)
                .unwrap_or("start");
            if state == "start" || state == "detect" {
                replies.push(WebSocketSessionReply::Text(format!(
                    r#"{{"session_id":"{}","type":"stt","text":"simulated user speech from SDKWork"}}"#,
                    json_escape(&session_id)
                )));
                replies.push(WebSocketSessionReply::Text(format!(
                    r#"{{"session_id":"{}","type":"llm","emotion":"happy","text":"connected"}}"#,
                    json_escape(&session_id)
                )));
                replies.push(WebSocketSessionReply::Text(format!(
                    r#"{{"session_id":"{}","type":"tts","state":"start"}}"#,
                    json_escape(&session_id)
                )));
            } else if state == "stop" {
                replies.push(WebSocketSessionReply::Text(format!(
                    r#"{{"session_id":"{}","type":"tts","state":"stop"}}"#,
                    json_escape(&session_id)
                )));
            }
        }
        "mcp" => {
            let kind = result
                .envelope
                .extensions
                .get("xiaozhi.mcp.kind")
                .map(String::as_str)
                .unwrap_or("payload");
            let inbound_json = std::str::from_utf8(&result.envelope.payload).ok();
            if let Some(outbound) = xiaozhi_simulator_mcp_reply(
                "websocket",
                &session_id,
                result.message.device_id.as_deref(),
                result.message.client_id.as_deref(),
                kind,
                inbound_json,
                result
                    .envelope
                    .extensions
                    .get("xiaozhi.mcp.id_json")
                    .map(String::as_str),
                result
                    .envelope
                    .extensions
                    .get("xiaozhi.mcp.method")
                    .map(String::as_str),
                mcp_tool_provider,
                mcp_tool_invoker,
                mcp_tool_policy,
            ) {
                replies.push(WebSocketSessionReply::Text(outbound));
            }
        }
        "audio" => {
            if frame.opcode == WebSocketOpcode::Binary {
                replies.push(WebSocketSessionReply::Text(format!(
                    r#"{{"session_id":"{}","type":"tts","state":"sentence_start","text":"received {} bytes of opus audio"}}"#,
                    json_escape(&session_id),
                    result.envelope.payload.len()
                )));
                replies.push(WebSocketSessionReply::Text(format!(
                    r#"{{"session_id":"{}","type":"tts","state":"stop"}}"#,
                    json_escape(&session_id)
                )));
            }
        }
        "abort" => {
            replies.push(WebSocketSessionReply::Text(format!(
                r#"{{"session_id":"{}","type":"tts","state":"stop"}}"#,
                json_escape(&session_id)
            )));
        }
        "goodbye" => {
            replies.push(WebSocketSessionReply::Text(format!(
                r#"{{"session_id":"{}","type":"goodbye"}}"#,
                json_escape(&session_id)
            )));
        }
        _ => {}
    }

    Ok(replies)
}

pub fn xiaozhi_mqtt_session_reply(
    server: &TransportServer,
    session: Option<&XiaozhiMqttUdpSession>,
    inbound_json: &str,
) -> Result<(MqttSessionReply, Option<XiaozhiMqttUdpSession>), TransportError> {
    let options = XiaozhiSessionOptions::from_env();
    xiaozhi_mqtt_session_reply_with_options(server, session, inbound_json, &options)
}

pub fn xiaozhi_mqtt_session_reply_with_options(
    server: &TransportServer,
    session: Option<&XiaozhiMqttUdpSession>,
    inbound_json: &str,
    options: &XiaozhiSessionOptions,
) -> Result<(MqttSessionReply, Option<XiaozhiMqttUdpSession>), TransportError> {
    let mcp_tool_provider = options.mcp_tool_provider();
    let mcp_tool_invoker = options.mcp_tool_invoker();
    let mcp_tool_policy = options.mcp_tool_policy();
    xiaozhi_mqtt_session_reply_with_mcp_tool_provider_and_invoker(
        server,
        session,
        inbound_json,
        mcp_tool_provider.as_ref(),
        mcp_tool_invoker.as_ref(),
        mcp_tool_policy.as_ref(),
    )
}

pub fn xiaozhi_mqtt_session_reply_with_mcp_tool_provider(
    server: &TransportServer,
    session: Option<&XiaozhiMqttUdpSession>,
    inbound_json: &str,
    mcp_tool_provider: &dyn XiaozhiSimulatorMcpToolProvider,
) -> Result<(MqttSessionReply, Option<XiaozhiMqttUdpSession>), TransportError> {
    let mcp_tool_invoker = DefaultXiaozhiSimulatorMcpToolInvoker;
    let mcp_tool_policy = AllowAllXiaozhiSimulatorMcpToolPolicy;
    xiaozhi_mqtt_session_reply_with_mcp_tool_provider_and_invoker(
        server,
        session,
        inbound_json,
        mcp_tool_provider,
        &mcp_tool_invoker,
        &mcp_tool_policy,
    )
}

pub fn xiaozhi_mqtt_session_reply_with_mcp_tool_provider_and_invoker(
    server: &TransportServer,
    session: Option<&XiaozhiMqttUdpSession>,
    inbound_json: &str,
    mcp_tool_provider: &dyn XiaozhiSimulatorMcpToolProvider,
    mcp_tool_invoker: &dyn XiaozhiSimulatorMcpToolInvoker,
    mcp_tool_policy: &dyn XiaozhiSimulatorMcpToolPolicy,
) -> Result<(MqttSessionReply, Option<XiaozhiMqttUdpSession>), TransportError> {
    let codec = match session {
        Some(session) => XiaozhiMqttCodec::new()
            .with_device_and_client(session.device_id.clone(), session.client_id.clone()),
        None => XiaozhiMqttCodec::new(),
    };

    let inbound = websocket_frame_to_inbound_frame(WebSocketFrame::text(inbound_json));
    let result = server
        .runtime
        .handle_inbound_frame_with_codec(XIAOZHI_MQTT_PATH, &codec, inbound)
        .map_err(TransportError::from_runtime_protocol)?;

    let mut next_session = session.cloned();
    let mut outbound = Vec::new();
    let mut close_audio_channel = false;

    match result.envelope.semantic_type.as_str() {
        "hello" => {
            let device_id = result
                .message
                .device_id
                .clone()
                .unwrap_or_else(|| "device".to_string());
            let client_id = result
                .message
                .client_id
                .clone()
                .unwrap_or_else(|| "client".to_string());
            let session_id = result
                .message
                .session_id
                .clone()
                .unwrap_or_else(|| format!("{device_id}-{client_id}"));

            let udp_server =
                env_string(ENV_XIAOZHI_MQTT_UDP_SERVER).unwrap_or_else(|| "127.0.0.1".to_string());
            let udp_port = env_u16(ENV_XIAOZHI_MQTT_UDP_PORT, 8888);
            let udp_key_hex = env_string(ENV_XIAOZHI_MQTT_UDP_KEY_HEX)
                .unwrap_or_else(|| "0123456789ABCDEF0123456789ABCDEF".to_string());
            let udp_nonce_hex = env_string(ENV_XIAOZHI_MQTT_UDP_NONCE_HEX)
                .unwrap_or_else(|| "01000000A1A2A3A40000000000000000".to_string());

            let hello = XiaozhiServerHello::mqtt_udp(
                session_id.clone(),
                udp_server.clone(),
                udp_port,
                udp_key_hex.clone(),
                udp_nonce_hex.clone(),
            )
            .with_audio_params(XiaozhiAudioParams::opus(24_000, 1, 60));
            outbound.push(xiaozhi_server_hello_response(hello));

            if result
                .envelope
                .extensions
                .get("xiaozhi.feature.mcp")
                .is_some_and(|value| value == "true")
            {
                outbound.push(format!(
                    r#"{{"session_id":"{}","type":"mcp","payload":{{"jsonrpc":"2.0","method":"initialize","params":{{"capabilities":{{"vision":{{"url":"http://localhost/iot/xiaozhi/vision","token":"simulator-token"}}}}}},"id":1}}}}"#,
                    json_escape(&session_id)
                ));
            }

            next_session = Some(XiaozhiMqttUdpSession {
                device_id,
                client_id,
                session_id,
                udp_server,
                udp_port,
                udp_key_hex,
                udp_nonce_hex,
                remote_sequence: 0,
            });
        }
        "listen" => {
            if let Some(session) = session {
                let state = result
                    .envelope
                    .extensions
                    .get("xiaozhi.listen.state")
                    .map(String::as_str)
                    .unwrap_or("start");
                if state == "start" || state == "detect" {
                    outbound.push(format!(
                        r#"{{"session_id":"{}","type":"stt","text":"simulated user speech from SDKWork"}}"#,
                        json_escape(&session.session_id)
                    ));
                    outbound.push(format!(
                        r#"{{"session_id":"{}","type":"llm","emotion":"happy","text":"connected"}}"#,
                        json_escape(&session.session_id)
                    ));
                    outbound.push(format!(
                        r#"{{"session_id":"{}","type":"tts","state":"start"}}"#,
                        json_escape(&session.session_id)
                    ));
                } else if state == "stop" {
                    outbound.push(format!(
                        r#"{{"session_id":"{}","type":"tts","state":"stop"}}"#,
                        json_escape(&session.session_id)
                    ));
                }
            }
        }
        "mcp" => {
            if let Some(session) = session {
                let kind = result
                    .envelope
                    .extensions
                    .get("xiaozhi.mcp.kind")
                    .map(String::as_str)
                    .unwrap_or("payload");
                let inbound_json = std::str::from_utf8(&result.envelope.payload).ok();
                if let Some(outbound_json) = xiaozhi_simulator_mcp_reply(
                    "mqtt",
                    &session.session_id,
                    Some(session.device_id.as_str()),
                    Some(session.client_id.as_str()),
                    kind,
                    inbound_json,
                    result
                        .envelope
                        .extensions
                        .get("xiaozhi.mcp.id_json")
                        .map(String::as_str),
                    result
                        .envelope
                        .extensions
                        .get("xiaozhi.mcp.method")
                        .map(String::as_str),
                    mcp_tool_provider,
                    mcp_tool_invoker,
                    mcp_tool_policy,
                ) {
                    outbound.push(outbound_json);
                }
            }
        }
        "abort" => {
            if let Some(session) = session {
                outbound.push(format!(
                    r#"{{"session_id":"{}","type":"tts","state":"stop"}}"#,
                    json_escape(&session.session_id)
                ));
            }
        }
        "goodbye" => {
            if let Some(session) = session {
                outbound.push(format!(
                    r#"{{"session_id":"{}","type":"goodbye"}}"#,
                    json_escape(&session.session_id)
                ));
            }
            close_audio_channel = true;
            next_session = None;
        }
        _ => {}
    }

    Ok((
        MqttSessionReply {
            outbound_json: outbound,
            close_audio_channel,
        },
        next_session,
    ))
}

pub fn xiaozhi_mqtt_udp_decode_audio(
    session: &XiaozhiMqttUdpSession,
    packet: &[u8],
) -> Result<XiaozhiUdpAudioPacket, TransportError> {
    let codec = session.udp_codec()?;
    codec
        .decode_audio_packet_with_min_sequence(packet, session.remote_sequence + 1)
        .map_err(|error| TransportError::new(error.code))
}

pub fn xiaozhi_codec_from_request(request: &HttpRequest) -> XiaozhiWebSocketCodec {
    let mut headers = Vec::new();
    if let Some(value) = request
        .header("authorization")
        .or_else(|| request.query_param("authorization"))
        .or_else(|| request.query_param("token"))
    {
        let value = if value.contains(' ') {
            value.to_string()
        } else {
            format!("Bearer {value}")
        };
        headers.push((AUTHORIZATION_HEADER.to_string(), value));
    }
    if let Some(value) = request
        .header("protocol-version")
        .or_else(|| request.query_param("protocol_version"))
        .or_else(|| request.query_param("version"))
    {
        headers.push((PROTOCOL_VERSION_HEADER.to_string(), value.to_string()));
    }
    if let Some(value) = request
        .header("device-id")
        .or_else(|| request.query_param("device_id"))
    {
        headers.push((DEVICE_ID_HEADER.to_string(), value.to_string()));
    }
    if let Some(value) = request
        .header("client-id")
        .or_else(|| request.query_param("client_id"))
    {
        headers.push((CLIENT_ID_HEADER.to_string(), value.to_string()));
    }

    XiaozhiWebSocketCodec::new().with_handshake_context(xiaozhi_handshake_context(headers))
}

fn xiaozhi_simulator_mcp_reply(
    transport: &str,
    session_id: &str,
    device_id: Option<&str>,
    client_id: Option<&str>,
    kind: &str,
    inbound_json: Option<&str>,
    id_json_hint: Option<&str>,
    method_hint: Option<&str>,
    mcp_tool_provider: &dyn XiaozhiSimulatorMcpToolProvider,
    mcp_tool_invoker: &dyn XiaozhiSimulatorMcpToolInvoker,
    mcp_tool_policy: &dyn XiaozhiSimulatorMcpToolPolicy,
) -> Option<String> {
    match kind {
        "notification" => None,
        "response" => Some(xiaozhi_mcp_tools_list_request(session_id, "", false, 2)),
        "payload" => None,
        "request" => {
            let request = parse_xiaozhi_mcp_request(inbound_json, id_json_hint, method_hint);
            if request.jsonrpc_version.as_deref() != Some("2.0") {
                return None;
            }
            if request.params_present && !request.params_is_object {
                return None;
            }
            if request.method.starts_with("notifications") {
                return None;
            }
            match request.method.as_str() {
                "initialize" => Some(xiaozhi_mcp_initialize_result(
                    session_id,
                    request.id_json.as_str(),
                )),
                "tools/list" => Some(xiaozhi_mcp_tools_list_result(
                    session_id,
                    request.id_json.as_str(),
                    request.cursor.as_str(),
                    request.with_user_tools,
                    mcp_tool_provider,
                )),
                "tools/call" => Some(xiaozhi_mcp_tools_call_result(
                    transport,
                    session_id,
                    device_id,
                    client_id,
                    request.id_json.as_str(),
                    request.params_is_object,
                    request.tool_name.as_deref(),
                    request.tool_arguments_present,
                    request.tool_arguments_is_object,
                    request.tool_arguments_json.as_deref(),
                    mcp_tool_provider,
                    mcp_tool_invoker,
                    mcp_tool_policy,
                )),
                method => Some(xiaozhi_mcp_error_response(
                    session_id,
                    request.id_json.as_str(),
                    &format!("Method not implemented: {method}"),
                )),
            }
        }
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct XiaozhiMcpRequest {
    jsonrpc_version: Option<String>,
    id_json: String,
    method: String,
    cursor: String,
    with_user_tools: bool,
    params_present: bool,
    params_is_object: bool,
    tool_name: Option<String>,
    tool_arguments_present: bool,
    tool_arguments_is_object: bool,
    tool_arguments_json: Option<String>,
}

fn parse_xiaozhi_mcp_request(
    inbound_json: Option<&str>,
    id_json_hint: Option<&str>,
    method_hint: Option<&str>,
) -> XiaozhiMcpRequest {
    let id_json_fallback = id_json_hint.unwrap_or("0").to_string();
    let method_fallback = method_hint.unwrap_or("").to_string();
    let Some(inbound_json) = inbound_json else {
        return XiaozhiMcpRequest {
            jsonrpc_version: None,
            id_json: id_json_fallback,
            method: method_fallback,
            cursor: String::new(),
            with_user_tools: false,
            params_present: false,
            params_is_object: false,
            tool_name: None,
            tool_arguments_present: false,
            tool_arguments_is_object: false,
            tool_arguments_json: None,
        };
    };

    let payload = json_object_field(inbound_json, "payload").unwrap_or(inbound_json);
    let jsonrpc_version = json_string_field(payload, "jsonrpc");
    let id_json = json_literal_field(payload, "id").unwrap_or(id_json_fallback);
    let method = if method_fallback.is_empty() {
        json_string_field(payload, "method").unwrap_or_default()
    } else {
        method_fallback
    };
    let params_raw = json_field_raw_value(payload, "params");
    let params_present = params_raw.is_some();
    let params = params_raw.and_then(|value| {
        let value = value.trim();
        if value.starts_with('{') && value.ends_with('}') {
            Some(value)
        } else {
            None
        }
    });
    let params_is_object = params.is_some();
    let cursor = params
        .and_then(|value| json_string_field(value, "cursor"))
        .unwrap_or_default();
    let with_user_tools = params
        .and_then(|value| json_scalar_field(value, "withUserTools"))
        .is_some_and(|value| value.eq_ignore_ascii_case("true"));
    let tool_name = params.and_then(|value| json_string_field(value, "name"));
    let tool_arguments_raw = params.and_then(|value| json_field_raw_value(value, "arguments"));
    let tool_arguments_present = tool_arguments_raw.is_some();
    let tool_arguments_json = tool_arguments_raw.and_then(|value| {
        let value = value.trim();
        if value.starts_with('{') && value.ends_with('}') {
            Some(value.to_string())
        } else {
            None
        }
    });
    let tool_arguments_is_object = tool_arguments_json.is_some();

    XiaozhiMcpRequest {
        jsonrpc_version,
        id_json,
        method,
        cursor,
        with_user_tools,
        params_present,
        params_is_object,
        tool_name,
        tool_arguments_present,
        tool_arguments_is_object,
        tool_arguments_json,
    }
}

fn xiaozhi_mcp_initialize_result(session_id: &str, id_json: &str) -> String {
    let payload = format!(
        r#"{{"jsonrpc":"2.0","id":{},"result":{{"protocolVersion":"{}","capabilities":{{"tools":{{}}}},"serverInfo":{{"name":"{}","version":"{}"}}}}}}"#,
        id_json,
        SIMULATOR_PROTOCOL_VERSION,
        SIMULATOR_SERVER_NAME,
        env!("CARGO_PKG_VERSION")
    );
    xiaozhi_mcp_wrap_payload(session_id, payload)
}

fn xiaozhi_mcp_tools_list_request(
    session_id: &str,
    cursor: &str,
    with_user_tools: bool,
    id: u32,
) -> String {
    let payload = format!(
        r#"{{"jsonrpc":"2.0","method":"tools/list","params":{{"cursor":"{}","withUserTools":{}}},"id":{}}}"#,
        json_escape(cursor),
        if with_user_tools { "true" } else { "false" },
        id
    );
    xiaozhi_mcp_wrap_payload(session_id, payload)
}

fn xiaozhi_mcp_tools_list_result(
    session_id: &str,
    id_json: &str,
    cursor: &str,
    with_user_tools: bool,
    mcp_tool_provider: &dyn XiaozhiSimulatorMcpToolProvider,
) -> String {
    let tools = mcp_tool_provider.tools();
    let page_size = env_u32(
        "SDKWORK_AIOT_XIAOZHI_MCP_PAGE_SIZE",
        DEFAULT_SIMULATOR_MCP_PAGE_SIZE as u32,
    ) as usize;
    let page_size = page_size.max(1);
    let mut found_cursor = cursor.is_empty();
    let mut page = Vec::new();
    let mut next_cursor: Option<String> = None;

    for tool in tools {
        if !found_cursor {
            if tool.name == cursor {
                found_cursor = true;
            } else {
                continue;
            }
        }
        if !with_user_tools && tool.user_only() {
            continue;
        }
        if page.len() >= page_size {
            next_cursor = Some(tool.name.clone());
            break;
        }
        page.push(tool);
    }

    if !found_cursor && !cursor.is_empty() {
        return xiaozhi_mcp_error_response(
            session_id,
            id_json,
            &format!("Unknown cursor: {cursor}"),
        );
    }

    let tools_json = page
        .iter()
        .map(|tool| {
            let mut out = format!(
                r#"{{"name":"{}","description":"{}","inputSchema":{}"#,
                json_escape(&tool.name),
                json_escape(&tool.description),
                tool.input_schema_json
            );
            if tool.user_only() {
                out.push_str(r#","annotations":{"audience":["user"]}"#);
            }
            out.push('}');
            out
        })
        .collect::<Vec<_>>()
        .join(",");

    let mut result = format!(r#"{{"tools":[{}]"#, tools_json);
    if let Some(next_cursor) = next_cursor {
        result.push_str(&format!(r#","nextCursor":"{}""#, json_escape(&next_cursor)));
    }
    result.push('}');

    let payload = format!(
        r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#,
        id_json, result
    );
    xiaozhi_mcp_wrap_payload(session_id, payload)
}

fn xiaozhi_mcp_tools_call_result(
    transport: &str,
    session_id: &str,
    device_id: Option<&str>,
    client_id: Option<&str>,
    id_json: &str,
    params_is_object: bool,
    tool_name: Option<&str>,
    tool_arguments_present: bool,
    tool_arguments_is_object: bool,
    tool_arguments_json: Option<&str>,
    mcp_tool_provider: &dyn XiaozhiSimulatorMcpToolProvider,
    mcp_tool_invoker: &dyn XiaozhiSimulatorMcpToolInvoker,
    mcp_tool_policy: &dyn XiaozhiSimulatorMcpToolPolicy,
) -> String {
    if !params_is_object {
        return xiaozhi_mcp_error_response(session_id, id_json, "Missing params");
    }
    let Some(tool_name) = tool_name else {
        return xiaozhi_mcp_error_response(session_id, id_json, "Missing name");
    };
    if tool_arguments_present && !tool_arguments_is_object {
        return xiaozhi_mcp_error_response(session_id, id_json, "Invalid arguments");
    }

    let tools = mcp_tool_provider.tools();
    let Some(tool) = tools.iter().find(|tool| tool.name == tool_name) else {
        return xiaozhi_mcp_error_response(
            session_id,
            id_json,
            &format!("Unknown tool: {tool_name}"),
        );
    };

    if let Some(argument_error) =
        validate_tool_arguments(tool.input_schema_json.as_str(), tool_arguments_json)
    {
        return xiaozhi_mcp_error_response(session_id, id_json, &argument_error);
    }

    let mut invocation_context = XiaozhiMcpInvocationContext::new(transport, session_id);
    if let Some(device_id) = device_id {
        invocation_context = invocation_context.with_device_id(device_id);
    }
    if let Some(client_id) = client_id {
        invocation_context = invocation_context.with_client_id(client_id);
    }

    let policy_evaluation =
        mcp_tool_policy.evaluate(&invocation_context, tool, tool_arguments_json);
    let should_log_policy_allow = env_bool(ENV_XIAOZHI_MCP_POLICY_LOG_ALLOW)
        || policy_evaluation.matched_rule_index.is_some();
    match policy_evaluation.decision {
        XiaozhiMcpPolicyDecision::Allow => {
            if should_log_policy_allow {
                eprintln!(
                    "{}",
                    mcp_policy_decision_log_line(
                        &invocation_context,
                        &tool.name,
                        &policy_evaluation,
                        tool_arguments_json.is_some(),
                    )
                );
            }
        }
        XiaozhiMcpPolicyDecision::Deny => {
            eprintln!(
                "{}",
                mcp_policy_decision_log_line(
                    &invocation_context,
                    &tool.name,
                    &policy_evaluation,
                    tool_arguments_json.is_some(),
                )
            );
            let error_message = policy_evaluation
                .error_message
                .as_deref()
                .unwrap_or("Tool not allowed by policy");
            return xiaozhi_mcp_error_response(session_id, id_json, error_message);
        }
    }

    let tool_response_text =
        match mcp_tool_invoker.invoke(&invocation_context, tool, tool_arguments_json) {
            Ok(text) => text,
            Err(error_message) => {
                return xiaozhi_mcp_error_response(session_id, id_json, &error_message)
            }
        };

    let payload = format!(
        r#"{{"jsonrpc":"2.0","id":{},"result":{{"content":[{{"type":"text","text":"{}"}}],"isError":false}}}}"#,
        id_json,
        json_escape(&tool_response_text)
    );
    xiaozhi_mcp_wrap_payload(session_id, payload)
}

fn xiaozhi_mcp_error_response(session_id: &str, id_json: &str, message: &str) -> String {
    let payload = format!(
        r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32601,"message":"{}"}}}}"#,
        id_json,
        json_escape(message)
    );
    xiaozhi_mcp_wrap_payload(session_id, payload)
}

fn mcp_policy_decision_log_line(
    context: &XiaozhiMcpInvocationContext,
    tool_name: &str,
    evaluation: &XiaozhiMcpPolicyEvaluation,
    arguments_present: bool,
) -> String {
    let decision = match evaluation.decision {
        XiaozhiMcpPolicyDecision::Allow => "allow",
        XiaozhiMcpPolicyDecision::Deny => "deny",
    };
    let rule_index = evaluation
        .matched_rule_index
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());
    let device_id = context.device_id.as_deref().unwrap_or("");
    let client_id = context.client_id.as_deref().unwrap_or("");
    let message = evaluation.error_message.as_deref().unwrap_or("");
    format!(
        "sdkwork-aiot-gateway mcp_policy_decision decision={} rule_index={} transport={} session_id={} device_id={} client_id={} tool={} arguments_present={} message=\"{}\"",
        decision,
        rule_index,
        context.transport,
        context.session_id,
        device_id,
        client_id,
        tool_name,
        arguments_present,
        json_escape(message)
    )
}

fn xiaozhi_mcp_wrap_payload(session_id: &str, payload: String) -> String {
    format!(
        r#"{{"session_id":"{}","type":"mcp","payload":{}}}"#,
        json_escape(session_id),
        payload
    )
}

fn built_in_simulator_mcp_tools() -> Vec<XiaozhiSimulatorMcpToolSpec> {
    vec![
        XiaozhiSimulatorMcpToolSpec::new(
            "self.get_device_status",
            "Get the current device status.",
            r#"{"type":"object","properties":{},"required":[]}"#,
        ),
        XiaozhiSimulatorMcpToolSpec::new(
            "self.audio_speaker.set_volume",
            "Set output volume from 0 to 100.",
            r#"{"type":"object","properties":{"volume":{"type":"integer","minimum":0,"maximum":100}},"required":["volume"]}"#,
        ),
        XiaozhiSimulatorMcpToolSpec::new(
            "self.screen.set_brightness",
            "Set screen brightness from 0 to 100.",
            r#"{"type":"object","properties":{"brightness":{"type":"integer","minimum":0,"maximum":100}},"required":["brightness"]}"#,
        ),
        XiaozhiSimulatorMcpToolSpec::new(
            "self.reboot",
            "Reboot the device.",
            r#"{"type":"object","properties":{},"required":[]}"#,
        )
        .with_user_only(true),
    ]
}

fn activation_challenge_registry_from_env() -> Arc<dyn XiaozhiActivationChallengeRegistry> {
    let kind = env_string(ENV_XIAOZHI_ACTIVATION_REGISTRY_KIND);
    if kind
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("redis"))
    {
        if let Some(redis_url) = env_string(ENV_XIAOZHI_ACTIVATION_REGISTRY_REDIS_URL) {
            return Arc::new(RedisXiaozhiActivationChallengeRegistry::new(
                redis_url,
                env_string(ENV_XIAOZHI_ACTIVATION_REGISTRY_REDIS_PREFIX),
            ));
        }
        eprintln!(
            "sdkwork-aiot-gateway activation_registry_redis_missing_url kind=redis fallback=in_memory"
        );
        return Arc::new(InMemoryXiaozhiActivationChallengeRegistry::new());
    }

    if let Some(path) = env_string(ENV_XIAOZHI_ACTIVATION_REGISTRY_PATH) {
        if kind
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case("sqlite"))
        {
            return Arc::new(SqliteXiaozhiActivationChallengeRegistry::new(
                PathBuf::from(path),
            ));
        }
        return Arc::new(FileBackedXiaozhiActivationChallengeRegistry::new(
            PathBuf::from(path),
        ));
    }
    Arc::new(InMemoryXiaozhiActivationChallengeRegistry::new())
}

fn validate_tool_arguments(
    input_schema_json: &str,
    tool_arguments_json: Option<&str>,
) -> Option<String> {
    let required_fields = json_array_strings(input_schema_json, "required");
    if required_fields.is_empty() {
        return None;
    }
    let properties_json = json_object_field(input_schema_json, "properties").unwrap_or("{}");
    let arguments_json = tool_arguments_json.unwrap_or("{}");
    for field in required_fields {
        let Some(property_schema) = json_object_field(properties_json, &field) else {
            continue;
        };
        if let Some(error_message) = tool_argument_error(arguments_json, &field, property_schema) {
            return Some(error_message);
        }
    }
    None
}

fn tool_argument_error(
    arguments_json: &str,
    field: &str,
    property_schema_json: &str,
) -> Option<String> {
    let Some(raw_value) = json_field_raw_value(arguments_json, field) else {
        return Some(format!("Missing valid argument: {field}"));
    };
    let field_type =
        json_string_field(property_schema_json, "type").unwrap_or_else(|| "string".to_string());
    match field_type.as_str() {
        "boolean" => {
            if is_json_boolean_literal(raw_value) {
                None
            } else {
                Some(format!("Missing valid argument: {field}"))
            }
        }
        "integer" => match parse_json_number_value(raw_value) {
            Some(value) => {
                let integer_value = truncate_to_i64(value);
                integer_value_schema_error(integer_value, property_schema_json)
            }
            None => Some(format!("Missing valid argument: {field}")),
        },
        "number" => {
            if parse_json_number_value(raw_value).is_some() {
                None
            } else {
                Some(format!("Missing valid argument: {field}"))
            }
        }
        "object" => {
            let value = raw_value.trim();
            if value.starts_with('{') && value.ends_with('}') {
                None
            } else {
                Some(format!("Missing valid argument: {field}"))
            }
        }
        "array" => {
            let value = raw_value.trim();
            if value.starts_with('[') && value.ends_with(']') {
                None
            } else {
                Some(format!("Missing valid argument: {field}"))
            }
        }
        _ => {
            if parse_json_string(raw_value).is_some() {
                None
            } else {
                Some(format!("Missing valid argument: {field}"))
            }
        }
    }
}

fn integer_value_schema_error(value: i64, property_schema_json: &str) -> Option<String> {
    if let Some(minimum) = json_scalar_field(property_schema_json, "minimum")
        .and_then(|value| value.parse::<i64>().ok())
    {
        if value < minimum {
            return Some(format!("Value is below minimum allowed: {minimum}"));
        }
    }
    if let Some(maximum) = json_scalar_field(property_schema_json, "maximum")
        .and_then(|value| value.parse::<i64>().ok())
    {
        if value > maximum {
            return Some(format!("Value exceeds maximum allowed: {maximum}"));
        }
    }
    None
}

fn parse_json_number_value(value: &str) -> Option<f64> {
    let literal = value.trim();
    if literal.is_empty() {
        return None;
    }
    literal
        .parse::<f64>()
        .ok()
        .filter(|parsed| parsed.is_finite())
}

fn parse_mcp_policy_rules(raw: &str) -> Vec<McpPolicyRule> {
    raw.split(';')
        .map(str::trim)
        .filter(|rule| !rule.is_empty())
        .filter_map(parse_mcp_policy_rule)
        .collect()
}

fn parse_mcp_policy_rule(rule: &str) -> Option<McpPolicyRule> {
    let mut parts = rule.split('|');
    let decision_raw = parts.next()?.trim().to_ascii_lowercase();
    let decision = match decision_raw.as_str() {
        "allow" => McpPolicyDecision::Allow,
        "deny" => McpPolicyDecision::Deny,
        _ => return None,
    };

    let mut pattern = McpPolicyRulePattern::default();
    for segment in parts {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        let Some((key, value)) = segment.split_once('=') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        if let Some(predicate) = parse_mcp_policy_numeric_argument_predicate(key.as_str(), value) {
            pattern.numeric_arg_predicates.push(predicate);
            continue;
        }
        if let Some(predicate) = parse_mcp_policy_string_argument_predicate(key.as_str(), value) {
            pattern.string_arg_predicates.push(predicate);
            continue;
        }
        if let Some(predicate) = parse_mcp_policy_boolean_argument_predicate(key.as_str(), value) {
            pattern.boolean_arg_predicates.push(predicate);
            continue;
        }
        match key.as_str() {
            "tool" => pattern.tool = Some(value.to_string()),
            "transport" => pattern.transport = Some(value.to_ascii_lowercase()),
            "device_prefix" => pattern.device_prefix = Some(value.to_string()),
            "client_prefix" => pattern.client_prefix = Some(value.to_string()),
            _ => {}
        }
    }
    Some(McpPolicyRule { decision, pattern })
}

fn mcp_policy_rule_matches(
    rule: &McpPolicyRule,
    context: &XiaozhiMcpInvocationContext,
    tool: &XiaozhiSimulatorMcpToolSpec,
    tool_arguments_json: Option<&str>,
) -> bool {
    if let Some(expected_tool) = &rule.pattern.tool {
        if tool.name != *expected_tool {
            return false;
        }
    }
    if let Some(expected_transport) = &rule.pattern.transport {
        if context.transport.to_ascii_lowercase() != *expected_transport {
            return false;
        }
    }
    if let Some(expected_prefix) = &rule.pattern.device_prefix {
        let Some(device_id) = context.device_id.as_deref() else {
            return false;
        };
        if !device_id.starts_with(expected_prefix) {
            return false;
        }
    }
    if let Some(expected_prefix) = &rule.pattern.client_prefix {
        let Some(client_id) = context.client_id.as_deref() else {
            return false;
        };
        if !client_id.starts_with(expected_prefix) {
            return false;
        }
    }
    if !mcp_policy_numeric_predicates_match(
        &rule.pattern.numeric_arg_predicates,
        tool_arguments_json,
    ) {
        return false;
    }
    if !mcp_policy_string_predicates_match(&rule.pattern.string_arg_predicates, tool_arguments_json)
    {
        return false;
    }
    if !mcp_policy_boolean_predicates_match(
        &rule.pattern.boolean_arg_predicates,
        tool_arguments_json,
    ) {
        return false;
    }
    true
}

fn parse_mcp_policy_numeric_argument_predicate(
    key: &str,
    value: &str,
) -> Option<McpPolicyNumericArgumentPredicate> {
    let (field, operator) = if let Some(field) = key
        .strip_prefix("arg_")
        .and_then(|tail| tail.strip_suffix("_gte"))
    {
        (field, McpPolicyNumericOperator::Gte)
    } else if let Some(field) = key
        .strip_prefix("arg_")
        .and_then(|tail| tail.strip_suffix("_lte"))
    {
        (field, McpPolicyNumericOperator::Lte)
    } else if let Some(field) = key
        .strip_prefix("arg_")
        .and_then(|tail| tail.strip_suffix("_gt"))
    {
        (field, McpPolicyNumericOperator::Gt)
    } else if let Some(field) = key
        .strip_prefix("arg_")
        .and_then(|tail| tail.strip_suffix("_lt"))
    {
        (field, McpPolicyNumericOperator::Lt)
    } else if let Some(field) = key
        .strip_prefix("arg_")
        .and_then(|tail| tail.strip_suffix("_eq"))
    {
        (field, McpPolicyNumericOperator::Eq)
    } else if let Some(field) = key
        .strip_prefix("arg_")
        .and_then(|tail| tail.strip_suffix("_ne"))
    {
        (field, McpPolicyNumericOperator::Ne)
    } else {
        return None;
    };

    if field.trim().is_empty() {
        return None;
    }
    let expected = value
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())?;
    Some(McpPolicyNumericArgumentPredicate {
        field: field.to_string(),
        operator,
        expected,
    })
}

fn mcp_policy_numeric_predicates_match(
    predicates: &[McpPolicyNumericArgumentPredicate],
    tool_arguments_json: Option<&str>,
) -> bool {
    if predicates.is_empty() {
        return true;
    }
    let arguments_json = tool_arguments_json.unwrap_or("{}");
    predicates.iter().all(|predicate| {
        let Some(actual_raw) = json_field_raw_value(arguments_json, &predicate.field) else {
            return false;
        };
        let Some(actual) = parse_json_number_value(actual_raw) else {
            return false;
        };
        mcp_policy_numeric_compare(actual, predicate.operator, predicate.expected)
    })
}

fn mcp_policy_numeric_compare(
    actual: f64,
    operator: McpPolicyNumericOperator,
    expected: f64,
) -> bool {
    const EPSILON: f64 = 1e-9;
    match operator {
        McpPolicyNumericOperator::Gte => actual >= expected,
        McpPolicyNumericOperator::Lte => actual <= expected,
        McpPolicyNumericOperator::Gt => actual > expected,
        McpPolicyNumericOperator::Lt => actual < expected,
        McpPolicyNumericOperator::Eq => (actual - expected).abs() <= EPSILON,
        McpPolicyNumericOperator::Ne => (actual - expected).abs() > EPSILON,
    }
}

fn parse_mcp_policy_string_argument_predicate(
    key: &str,
    value: &str,
) -> Option<McpPolicyStringArgumentPredicate> {
    let (field, operator) = if let Some(field) = key
        .strip_prefix("arg_")
        .and_then(|tail| tail.strip_suffix("_str_eq"))
    {
        (field, McpPolicyStringOperator::Eq)
    } else if let Some(field) = key
        .strip_prefix("arg_")
        .and_then(|tail| tail.strip_suffix("_str_ne"))
    {
        (field, McpPolicyStringOperator::Ne)
    } else if let Some(field) = key
        .strip_prefix("arg_")
        .and_then(|tail| tail.strip_suffix("_str_prefix"))
    {
        (field, McpPolicyStringOperator::Prefix)
    } else {
        return None;
    };
    if field.trim().is_empty() {
        return None;
    }
    Some(McpPolicyStringArgumentPredicate {
        field: field.to_string(),
        operator,
        expected: value.to_string(),
    })
}

fn mcp_policy_string_predicates_match(
    predicates: &[McpPolicyStringArgumentPredicate],
    tool_arguments_json: Option<&str>,
) -> bool {
    if predicates.is_empty() {
        return true;
    }
    let arguments_json = tool_arguments_json.unwrap_or("{}");
    predicates.iter().all(|predicate| {
        let Some(actual_raw) = json_field_raw_value(arguments_json, &predicate.field) else {
            return false;
        };
        let Some(actual) = parse_json_string(actual_raw) else {
            return false;
        };
        match predicate.operator {
            McpPolicyStringOperator::Eq => actual == predicate.expected,
            McpPolicyStringOperator::Ne => actual != predicate.expected,
            McpPolicyStringOperator::Prefix => actual.starts_with(predicate.expected.as_str()),
        }
    })
}

fn parse_mcp_policy_boolean_argument_predicate(
    key: &str,
    value: &str,
) -> Option<McpPolicyBooleanArgumentPredicate> {
    let (field, operator) = if let Some(field) = key
        .strip_prefix("arg_")
        .and_then(|tail| tail.strip_suffix("_bool_eq"))
    {
        (field, McpPolicyBooleanOperator::Eq)
    } else if let Some(field) = key
        .strip_prefix("arg_")
        .and_then(|tail| tail.strip_suffix("_bool_ne"))
    {
        (field, McpPolicyBooleanOperator::Ne)
    } else {
        return None;
    };
    if field.trim().is_empty() {
        return None;
    }
    let expected = match value.to_ascii_lowercase().as_str() {
        "true" => true,
        "false" => false,
        _ => return None,
    };
    Some(McpPolicyBooleanArgumentPredicate {
        field: field.to_string(),
        operator,
        expected,
    })
}

fn mcp_policy_boolean_predicates_match(
    predicates: &[McpPolicyBooleanArgumentPredicate],
    tool_arguments_json: Option<&str>,
) -> bool {
    if predicates.is_empty() {
        return true;
    }
    let arguments_json = tool_arguments_json.unwrap_or("{}");
    predicates.iter().all(|predicate| {
        let Some(actual_raw) = json_field_raw_value(arguments_json, &predicate.field) else {
            return false;
        };
        let actual = match actual_raw.trim() {
            "true" => true,
            "false" => false,
            _ => return false,
        };
        match predicate.operator {
            McpPolicyBooleanOperator::Eq => actual == predicate.expected,
            McpPolicyBooleanOperator::Ne => actual != predicate.expected,
        }
    })
}

fn truncate_to_i64(value: f64) -> i64 {
    if value >= i64::MAX as f64 {
        i64::MAX
    } else if value <= i64::MIN as f64 {
        i64::MIN
    } else {
        value as i64
    }
}

fn is_json_boolean_literal(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed == "true" || trimmed == "false"
}

fn register_challenge_in_entries(
    entries: &mut HashMap<ActivationChallengeKey, ActivationChallengeEntry>,
    request: &HttpRequest,
    challenge: &str,
    timeout_ms: u32,
) -> u64 {
    let now = current_unix_time_millis();
    let expires_at_millis = now.saturating_add(i64::from(timeout_ms));
    let key = activation_challenge_key(request, challenge);
    entries.insert(key, ActivationChallengeEntry { expires_at_millis });
    let before_retain = entries.len();
    entries.retain(|_, entry| now < entry.expires_at_millis);
    before_retain.saturating_sub(entries.len()) as u64
}

fn consume_challenge_in_entries(
    entries: &mut HashMap<ActivationChallengeKey, ActivationChallengeEntry>,
    request: &HttpRequest,
    challenge: &str,
) -> (bool, u64) {
    let now = current_unix_time_millis();
    let key = activation_challenge_key(request, challenge);
    let before_retain = entries.len();
    entries.retain(|_, entry| now < entry.expires_at_millis);
    let pruned = before_retain.saturating_sub(entries.len()) as u64;
    let Some(entry) = entries.get(&key) else {
        return (false, pruned);
    };
    if now >= entry.expires_at_millis {
        entries.remove(&key);
        return (false, pruned);
    }
    entries.remove(&key);
    (true, pruned)
}

fn activation_registry_records(
    entries: &HashMap<ActivationChallengeKey, ActivationChallengeEntry>,
) -> Vec<ActivationRegistryRecord> {
    let mut records = entries
        .iter()
        .map(|(key, entry)| ActivationRegistryRecord {
            key: key.clone(),
            entry: entry.clone(),
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| {
        (
            &left.key.device_id,
            &left.key.client_id,
            &left.key.challenge,
            left.entry.expires_at_millis,
        )
            .cmp(&(
                &right.key.device_id,
                &right.key.client_id,
                &right.key.challenge,
                right.entry.expires_at_millis,
            ))
    });
    records
}

fn load_activation_registry_entries(
    path: &Path,
) -> HashMap<ActivationChallengeKey, ActivationChallengeEntry> {
    let mut entries = HashMap::new();
    let Ok(raw) = fs::read_to_string(path) else {
        return entries;
    };
    let now = current_unix_time_millis();
    for line in raw.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let mut parts = line.split('\t');
        let Some(expires_raw) = parts.next() else {
            continue;
        };
        let Some(device_hex) = parts.next() else {
            continue;
        };
        let Some(client_hex) = parts.next() else {
            continue;
        };
        let Some(challenge_hex) = parts.next() else {
            continue;
        };
        let Ok(expires_at_millis) = expires_raw.parse::<i64>() else {
            continue;
        };
        if expires_at_millis <= now {
            continue;
        }
        let Some(device_id) = decode_registry_hex(device_hex) else {
            continue;
        };
        let Some(client_id) = decode_registry_hex(client_hex) else {
            continue;
        };
        let Some(challenge) = decode_registry_hex(challenge_hex) else {
            continue;
        };
        let key = ActivationChallengeKey {
            device_id,
            client_id,
            challenge,
        };
        entries.insert(key, ActivationChallengeEntry { expires_at_millis });
    }
    entries
}

fn write_activation_registry_records(
    path: &Path,
    records: &[ActivationRegistryRecord],
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut out = String::new();
    for record in records {
        out.push_str(&record.entry.expires_at_millis.to_string());
        out.push('\t');
        out.push_str(&encode_registry_hex(&record.key.device_id));
        out.push('\t');
        out.push_str(&encode_registry_hex(&record.key.client_id));
        out.push('\t');
        out.push_str(&encode_registry_hex(&record.key.challenge));
        out.push('\n');
    }
    fs::write(path, out)
}

struct ActivationRegistryFileLockGuard {
    lock_path: PathBuf,
}

impl Drop for ActivationRegistryFileLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

fn with_activation_registry_file_lock<T>(
    lock_path: &Path,
    wait_timeout: Duration,
    poll_interval: Duration,
    stale_after: Duration,
    action: impl FnOnce() -> io::Result<T>,
) -> io::Result<T> {
    if let Some(parent) = lock_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let start_millis = current_unix_time_millis();
    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(_file) => {
                let _guard = ActivationRegistryFileLockGuard {
                    lock_path: lock_path.to_path_buf(),
                };
                return action();
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                if is_activation_registry_lock_stale(lock_path, stale_after) {
                    let _ = fs::remove_file(lock_path);
                    continue;
                }

                let now = current_unix_time_millis();
                let elapsed = now.saturating_sub(start_millis).max(0) as u64;
                if elapsed >= duration_millis_u64(wait_timeout) {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        format!(
                            "activation registry lock timeout path={} wait_ms={}",
                            lock_path.display(),
                            duration_millis_u64(wait_timeout)
                        ),
                    ));
                }
                std::thread::sleep(poll_interval);
            }
            Err(error) => return Err(error),
        }
    }
}

fn is_activation_registry_lock_stale(lock_path: &Path, stale_after: Duration) -> bool {
    if duration_millis_u64(stale_after) == 0 {
        return false;
    }
    let Ok(metadata) = fs::metadata(lock_path) else {
        return false;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return false;
    };
    age >= stale_after
}

fn activation_registry_lock_path(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".lock");
    PathBuf::from(os)
}

fn read_simulator_mcp_tools_file(path: &Path) -> Option<Vec<XiaozhiSimulatorMcpToolSpec>> {
    let raw = fs::read_to_string(path).ok()?;
    let tools_json = json_array_field(&raw, "tools").or_else(|| {
        let trimmed = raw.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            Some(trimmed)
        } else {
            None
        }
    })?;
    let objects = json_array_objects(tools_json);
    if objects.is_empty() {
        return None;
    }

    let mut tools = Vec::new();
    for object in objects {
        let Some(name) = json_string_field(object, "name") else {
            continue;
        };
        if name.trim().is_empty() {
            continue;
        }
        let description = json_string_field(object, "description").unwrap_or_default();
        let input_schema_json = json_object_field(object, "inputSchema")
            .map(str::to_string)
            .unwrap_or_else(|| r#"{"type":"object","properties":{},"required":[]}"#.to_string());
        let user_only = json_scalar_field(object, "userOnly")
            .is_some_and(|value| value.eq_ignore_ascii_case("true"));
        let result_text = json_string_field(object, "resultText");
        let mut tool = XiaozhiSimulatorMcpToolSpec::new(name, description, input_schema_json)
            .with_user_only(user_only);
        if let Some(result_text) = result_text {
            tool = tool.with_simulated_result_text(result_text);
        }
        tools.push(tool);
    }

    if tools.is_empty() {
        None
    } else {
        Some(tools)
    }
}

fn encode_registry_hex(value: &str) -> String {
    let mut out = String::with_capacity(value.len() * 2);
    for byte in value.as_bytes() {
        out.push(hex_nibble(byte >> 4));
        out.push(hex_nibble(byte & 0x0f));
    }
    out
}

fn decode_registry_hex(value: &str) -> Option<String> {
    if value.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(value.len() / 2);
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let hi = hex_value(bytes[index])?;
        let lo = hex_value(bytes[index + 1])?;
        out.push((hi << 4) | lo);
        index += 2;
    }
    String::from_utf8(out).ok()
}

fn hex_nibble(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + (value - 10)),
        _ => '0',
    }
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(10 + (value - b'a')),
        b'A'..=b'F' => Some(10 + (value - b'A')),
        _ => None,
    }
}

fn websocket_scheme(request: &HttpRequest) -> &'static str {
    match request.header("x-forwarded-proto") {
        Some(proto) if proto.eq_ignore_ascii_case("http") => "ws",
        Some(proto) if proto.eq_ignore_ascii_case("https") => "wss",
        _ if is_local_host(request.header("host").unwrap_or_default()) => "ws",
        _ => "wss",
    }
}

fn register_activation_challenge_if_present(
    challenge_registry: &dyn XiaozhiActivationChallengeRegistry,
    request: &HttpRequest,
    metadata: &XiaozhiOtaMetadata,
) {
    let Some(activation) = metadata.activation.as_ref() else {
        return;
    };
    let Some(challenge) = activation.challenge.as_deref() else {
        return;
    };

    challenge_registry.register_challenge(request, challenge, activation.timeout_ms);
}

fn activation_challenge_key(request: &HttpRequest, challenge: &str) -> ActivationChallengeKey {
    let device_id = request
        .header("device-id")
        .unwrap_or_default()
        .trim()
        .to_string();
    let client_id = request
        .header("client-id")
        .unwrap_or_default()
        .trim()
        .to_string();
    ActivationChallengeKey {
        device_id,
        client_id,
        challenge: challenge.to_string(),
    }
}

fn activation_request_accepted(
    request: &HttpRequest,
    challenge_registry: Option<&dyn XiaozhiActivationChallengeRegistry>,
) -> bool {
    if env_bool(ENV_XIAOZHI_ACTIVATE_AUTO_ACCEPT) {
        return true;
    }

    let Some(body) = std::str::from_utf8(&request.body).ok() else {
        return false;
    };
    let Some(challenge) = json_string_field(body, "challenge") else {
        return false;
    };
    let Some(hmac) = json_string_field(body, "hmac") else {
        return false;
    };

    if activation_request_requires_strict_v2_validation(request) {
        let Some(algorithm) = json_string_field(body, "algorithm") else {
            return false;
        };
        if !algorithm.eq_ignore_ascii_case("hmac-sha256") {
            return false;
        }
        let Some(payload_serial_number) = json_string_field(body, "serial_number") else {
            return false;
        };
        let Some(header_serial_number) = request.header("serial-number") else {
            return false;
        };
        if !payload_serial_number.eq_ignore_ascii_case(header_serial_number.trim()) {
            return false;
        }
    }

    if let Some(expected) = env_string(ENV_XIAOZHI_ACTIVATE_EXPECTED_CHALLENGE) {
        if challenge != expected {
            return false;
        }
    }
    if let Some(expected) = env_string(ENV_XIAOZHI_ACTIVATE_EXPECTED_HMAC) {
        if hmac != expected {
            return false;
        }
    }

    if let Some(challenge_registry) = challenge_registry {
        challenge_registry.consume_challenge(request, &challenge)
    } else {
        true
    }
}

fn activation_request_requires_strict_v2_validation(request: &HttpRequest) -> bool {
    if !env_bool(ENV_XIAOZHI_ACTIVATE_STRICT_V2) {
        return false;
    }

    request
        .header("activation-version")
        .is_some_and(|value| value.trim() == "2")
}

fn mqtt_ota_from_env() -> Option<(String, String, String, String, String, String)> {
    Some((
        env_string(ENV_XIAOZHI_MQTT_ENDPOINT)?,
        env_string(ENV_XIAOZHI_MQTT_CLIENT_ID)?,
        env_string(ENV_XIAOZHI_MQTT_USERNAME)?,
        env_string(ENV_XIAOZHI_MQTT_PASSWORD)?,
        env_string(ENV_XIAOZHI_MQTT_PUBLISH_TOPIC)?,
        env_string(ENV_XIAOZHI_MQTT_SUBSCRIBE_TOPIC)?,
    ))
}

fn mqtt_udp_profile_from_env() -> Option<(String, u16, String, String)> {
    let server = env_string(ENV_XIAOZHI_MQTT_UDP_SERVER)?;
    let port = env_string(ENV_XIAOZHI_MQTT_UDP_PORT).and_then(|value| value.parse::<u16>().ok())?;
    let key_hex = env_string(ENV_XIAOZHI_MQTT_UDP_KEY_HEX)?;
    let nonce_hex = env_string(ENV_XIAOZHI_MQTT_UDP_NONCE_HEX)?;
    Some((server, port, key_hex, nonce_hex))
}

fn firmware_ota_from_env() -> Option<(String, String, bool)> {
    Some((
        env_string(ENV_XIAOZHI_FIRMWARE_VERSION)?,
        env_string(ENV_XIAOZHI_FIRMWARE_URL)?,
        env_bool(ENV_XIAOZHI_FIRMWARE_FORCE),
    ))
}

fn activation_profile_from_env() -> Option<(String, Option<String>, Option<String>, u32)> {
    let message = env_string(ENV_XIAOZHI_ACTIVATION_MESSAGE)
        .unwrap_or_else(|| DEFAULT_ACTIVATION_MESSAGE.to_string());
    let code = env_string(ENV_XIAOZHI_ACTIVATION_CODE);
    let challenge = env_string(ENV_XIAOZHI_ACTIVATION_CHALLENGE);
    let timeout_ms = env_u32(
        ENV_XIAOZHI_ACTIVATION_TIMEOUT_MS,
        DEFAULT_ACTIVATION_TIMEOUT_MS,
    );

    if code.is_none() && challenge.is_none() {
        None
    } else {
        Some((message, code, challenge, timeout_ms))
    }
}

fn env_string(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_u32(name: &str, default: u32) -> u32 {
    env_string(name)
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    env_string(name)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

fn env_u16(name: &str, default: u16) -> u16 {
    env_string(name)
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(default)
}

fn env_i32(name: &str, default: i32) -> i32 {
    env_string(name)
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(default)
}

fn env_bool(name: &str) -> bool {
    env_string(name).is_some_and(|value| {
        value == "1"
            || value.eq_ignore_ascii_case("true")
            || value.eq_ignore_ascii_case("yes")
            || value.eq_ignore_ascii_case("on")
    })
}

fn is_local_host(host: &str) -> bool {
    let host = host
        .strip_prefix('[')
        .and_then(|value| value.split_once(']').map(|(inner, _)| inner))
        .unwrap_or_else(|| host.split_once(':').map(|(name, _)| name).unwrap_or(host));

    matches!(host, "localhost" | "127.0.0.1" | "::1")
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

fn activation_registry_mark_backend(backend_code: u64) {
    ACTIVATION_REGISTRY_STATS
        .backend_kind
        .store(backend_code, Ordering::Relaxed);
}

fn activation_registry_backend_name(backend_code: u64) -> &'static str {
    match backend_code {
        ACTIVATION_REGISTRY_BACKEND_IN_MEMORY => "in_memory",
        ACTIVATION_REGISTRY_BACKEND_FILE => "file",
        ACTIVATION_REGISTRY_BACKEND_SQLITE => "sqlite",
        ACTIVATION_REGISTRY_BACKEND_REDIS => "redis",
        _ => "unknown",
    }
}

fn activation_registry_add_pruned(pruned: u64) {
    if pruned > 0 {
        ACTIVATION_REGISTRY_STATS
            .pruned_entries
            .fetch_add(pruned, Ordering::Relaxed);
    }
}

fn activation_registry_record_consume_outcome(consumed: bool) {
    if consumed {
        ACTIVATION_REGISTRY_STATS
            .consume_hits
            .fetch_add(1, Ordering::Relaxed);
    } else {
        ACTIVATION_REGISTRY_STATS
            .consume_misses
            .fetch_add(1, Ordering::Relaxed);
    }
}

fn sqlite_connection_for_path(path: &Path) -> Arc<Mutex<rusqlite::Connection>> {
    SQLITE_CONNECTION_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        if let Some(existing) = registry.get(path) {
            return Arc::clone(existing);
        }
        let connection = rusqlite::Connection::open(path).unwrap_or_else(|error| {
            panic!(
                "failed to open sqlite activation registry at {}: {error}",
                path.display()
            )
        });
        let connection = Arc::new(Mutex::new(connection));
        registry.insert(path.to_path_buf(), Arc::clone(&connection));
        connection
    })
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

fn json_string_field(input: &str, field: &str) -> Option<String> {
    let (start, end) = json_field_value_range(input, field)?;
    let value = input[start..end].trim();
    parse_json_string(value)
}

fn json_scalar_field(input: &str, field: &str) -> Option<String> {
    let (start, end) = json_field_value_range(input, field)?;
    let value = input[start..end].trim();
    if value.is_empty() {
        return None;
    }
    if value.starts_with('"') {
        parse_json_string(value)
    } else {
        Some(value.to_string())
    }
}

fn json_literal_field(input: &str, field: &str) -> Option<String> {
    let (start, end) = json_field_value_range(input, field)?;
    let value = input[start..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn json_field_raw_value<'a>(input: &'a str, field: &str) -> Option<&'a str> {
    let (start, end) = json_field_value_range(input, field)?;
    let value = input[start..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn json_object_field<'a>(input: &'a str, field: &str) -> Option<&'a str> {
    let (start, end) = json_field_value_range(input, field)?;
    let value = input[start..end].trim();
    if value.starts_with('{') && value.ends_with('}') {
        Some(value)
    } else {
        None
    }
}

fn json_array_strings(input: &str, field: &str) -> Vec<String> {
    let Some(array) = json_array_field(input, field) else {
        return Vec::new();
    };
    json_array_string_values(array)
}

fn json_array_string_values(input: &str) -> Vec<String> {
    let trimmed = input.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut cursor = 1usize;
    let bytes = trimmed.as_bytes();
    let end = bytes.len().saturating_sub(1);

    while cursor < end {
        cursor = skip_json_ws(trimmed, cursor);
        if cursor >= end {
            break;
        }
        if bytes[cursor] == b',' {
            cursor += 1;
            continue;
        }
        let Some(value_end) = json_value_end(trimmed, cursor) else {
            break;
        };
        let raw = trimmed[cursor..value_end].trim();
        if let Some(value) = parse_json_string(raw) {
            out.push(value);
        }
        cursor = value_end;
    }

    out
}

fn json_array_field<'a>(input: &'a str, field: &str) -> Option<&'a str> {
    let (start, end) = json_field_value_range(input, field)?;
    let value = input[start..end].trim();
    if value.starts_with('[') && value.ends_with(']') {
        Some(value)
    } else {
        None
    }
}

fn json_array_objects<'a>(input: &'a str) -> Vec<&'a str> {
    let trimmed = input.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut cursor = 1usize;
    let bytes = trimmed.as_bytes();
    let end = bytes.len().saturating_sub(1);

    while cursor < end {
        cursor = skip_json_ws(trimmed, cursor);
        if cursor >= end {
            break;
        }
        if bytes[cursor] == b',' {
            cursor += 1;
            continue;
        }
        if bytes[cursor] != b'{' {
            break;
        }

        let Some(value_end) = json_value_end(trimmed, cursor) else {
            break;
        };
        let value = trimmed[cursor..value_end].trim();
        if value.starts_with('{') && value.ends_with('}') {
            out.push(value);
        }
        cursor = value_end;
    }

    out
}

fn json_field_value_range(input: &str, field: &str) -> Option<(usize, usize)> {
    let key = format!("\"{field}\"");
    let index = input.find(&key)?;
    let mut cursor = skip_json_ws(input, index + key.len());
    if input.as_bytes().get(cursor).copied()? != b':' {
        return None;
    }
    cursor = skip_json_ws(input, cursor + 1);
    let end = json_value_end(input, cursor)?;
    Some((cursor, end))
}

fn parse_json_string(input: &str) -> Option<String> {
    if !input.starts_with('"') {
        return None;
    }
    let mut escaped = false;
    let mut out = String::new();
    for ch in input[1..].chars() {
        if escaped {
            let decoded = match ch {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{08}',
                'f' => '\u{0c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => ch,
            };
            out.push(decoded);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            other => out.push(other),
        }
    }
    None
}

fn skip_json_ws(input: &str, mut cursor: usize) -> usize {
    while input
        .as_bytes()
        .get(cursor)
        .is_some_and(u8::is_ascii_whitespace)
    {
        cursor += 1;
    }
    cursor
}

fn json_value_end(input: &str, start: usize) -> Option<usize> {
    let first = input.as_bytes().get(start).copied()?;
    match first {
        b'"' => {
            let mut escaped = false;
            for (offset, byte) in input.as_bytes()[start + 1..].iter().copied().enumerate() {
                if escaped {
                    escaped = false;
                    continue;
                }
                if byte == b'\\' {
                    escaped = true;
                    continue;
                }
                if byte == b'"' {
                    return Some(start + 2 + offset);
                }
            }
            None
        }
        b'{' => json_composite_end(input, start, b'{', b'}'),
        b'[' => json_composite_end(input, start, b'[', b']'),
        _ => {
            let rest = &input[start..];
            let offset = rest
                .find(|ch: char| ch == ',' || ch == '}' || ch == ']' || ch.is_whitespace())
                .unwrap_or(rest.len());
            Some(start + offset)
        }
    }
}

fn json_composite_end(input: &str, start: usize, open: u8, close: u8) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.get(start).copied()? != open {
        return None;
    }

    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, byte) in bytes[start..].iter().copied().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }

        match byte {
            b'"' => in_string = true,
            value if value == open => depth += 1,
            value if value == close => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(start + offset + 1);
                }
            }
            _ => {}
        }
    }

    None
}

fn problem_response(status: HttpStatus, code: &str) -> HttpResponse {
    HttpResponse::new(status)
        .with_header("content-type", "application/problem+json")
        .with_body(format!(
            r#"{{"type":"about:blank","title":"{}","status":{},"code":"{}"}}"#,
            status.reason(),
            status.code(),
            code
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_backed_activation_registry_round_trip_and_consume_once() {
        let _lock = lock_activation_registry_stats_for_test();
        let path = unique_test_file_path("activation-registry");
        let registry = FileBackedXiaozhiActivationChallengeRegistry::new(path.clone());
        let request = HttpRequest::new("POST", "/iot/xiaozhi/ota")
            .with_header("device-id", "device-01")
            .with_header("client-id", "client-01");

        registry.register_challenge(&request, "challenge-01", 60_000);
        assert!(registry.consume_challenge(&request, "challenge-01"));
        assert!(!registry.consume_challenge(&request, "challenge-01"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn file_backed_activation_registry_reload_supports_restart_semantics() {
        let _lock = lock_activation_registry_stats_for_test();
        let path = unique_test_file_path("activation-registry-reload");
        let request = HttpRequest::new("POST", "/iot/xiaozhi/ota")
            .with_header("device-id", "device-02")
            .with_header("client-id", "client-02");

        let registry = FileBackedXiaozhiActivationChallengeRegistry::new(path.clone());
        registry.register_challenge(&request, "challenge-02", 60_000);
        drop(registry);

        let reloaded = FileBackedXiaozhiActivationChallengeRegistry::new(path.clone());
        assert!(reloaded.consume_challenge(&request, "challenge-02"));
        assert!(!reloaded.consume_challenge(&request, "challenge-02"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn simulator_mcp_tools_file_parser_supports_object_and_array_root() {
        let object_path = unique_test_file_path("mcp-tools-object");
        fs::write(
            &object_path,
            r#"{"tools":[{"name":"self.a","description":"A","inputSchema":{"type":"object","properties":{},"required":[]},"userOnly":false}]}"#,
        )
        .expect("write object tools");
        let object_tools = read_simulator_mcp_tools_file(&object_path).expect("object tools");
        assert_eq!(object_tools.len(), 1);
        assert_eq!(object_tools[0].name, "self.a");
        assert!(!object_tools[0].user_only());

        let array_path = unique_test_file_path("mcp-tools-array");
        fs::write(
            &array_path,
            r#"[{"name":"self.b","description":"B","inputSchema":{"type":"object","properties":{},"required":[]},"userOnly":true}]"#,
        )
        .expect("write array tools");
        let array_tools = read_simulator_mcp_tools_file(&array_path).expect("array tools");
        assert_eq!(array_tools.len(), 1);
        assert_eq!(array_tools[0].name, "self.b");
        assert!(array_tools[0].user_only());

        let _ = fs::remove_file(object_path);
        let _ = fs::remove_file(array_path);
    }

    #[test]
    fn activation_registry_lock_path_appends_lock_suffix() {
        let data_path = PathBuf::from("xiaozhi-activation.state");
        let lock_path = activation_registry_lock_path(&data_path);
        assert_eq!(lock_path.to_string_lossy(), "xiaozhi-activation.state.lock");
    }

    #[test]
    fn stale_lock_is_detected_after_threshold() {
        let lock_path = unique_test_file_path("activation-lock-stale");
        fs::write(&lock_path, "lock").expect("write lock file");
        std::thread::sleep(Duration::from_millis(10));
        assert!(is_activation_registry_lock_stale(
            &lock_path,
            Duration::from_millis(1)
        ));
        let _ = fs::remove_file(lock_path);
    }

    #[test]
    fn file_backed_registry_keeps_both_entries_across_two_instances() {
        let _lock = lock_activation_registry_stats_for_test();
        let path = unique_test_file_path("activation-registry-two-instances");
        let request_a = HttpRequest::new("POST", "/iot/xiaozhi/ota")
            .with_header("device-id", "device-a")
            .with_header("client-id", "client-a");
        let request_b = HttpRequest::new("POST", "/iot/xiaozhi/ota")
            .with_header("device-id", "device-b")
            .with_header("client-id", "client-b");

        let registry_a = FileBackedXiaozhiActivationChallengeRegistry::with_locking(
            path.clone(),
            Duration::from_millis(500),
            Duration::from_millis(5),
            Duration::from_millis(2_000),
        );
        let registry_b = FileBackedXiaozhiActivationChallengeRegistry::with_locking(
            path.clone(),
            Duration::from_millis(500),
            Duration::from_millis(5),
            Duration::from_millis(2_000),
        );

        registry_a.register_challenge(&request_a, "challenge-shared", 60_000);
        registry_b.register_challenge(&request_b, "challenge-shared", 60_000);

        assert!(registry_a.consume_challenge(&request_a, "challenge-shared"));
        assert!(registry_b.consume_challenge(&request_b, "challenge-shared"));
        assert!(!registry_a.consume_challenge(&request_a, "challenge-shared"));
        assert!(!registry_b.consume_challenge(&request_b, "challenge-shared"));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(activation_registry_lock_path(&path));
    }

    #[test]
    fn sqlite_activation_registry_round_trip_and_consume_once() {
        let _lock = lock_activation_registry_stats_for_test();
        let path = unique_test_file_path("activation-registry-sqlite");
        let registry = SqliteXiaozhiActivationChallengeRegistry::new(path.clone());
        let request = HttpRequest::new("POST", "/iot/xiaozhi/ota")
            .with_header("device-id", "device-sqlite-01")
            .with_header("client-id", "client-sqlite-01");

        registry.register_challenge(&request, "challenge-sqlite-01", 60_000);
        assert!(registry.consume_challenge(&request, "challenge-sqlite-01"));
        assert!(!registry.consume_challenge(&request, "challenge-sqlite-01"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn sqlite_activation_registry_reload_supports_restart_semantics() {
        let _lock = lock_activation_registry_stats_for_test();
        let path = unique_test_file_path("activation-registry-sqlite-reload");
        let request = HttpRequest::new("POST", "/iot/xiaozhi/ota")
            .with_header("device-id", "device-sqlite-02")
            .with_header("client-id", "client-sqlite-02");

        let registry = SqliteXiaozhiActivationChallengeRegistry::new(path.clone());
        registry.register_challenge(&request, "challenge-sqlite-02", 60_000);
        drop(registry);

        let reloaded = SqliteXiaozhiActivationChallengeRegistry::new(path.clone());
        assert!(reloaded.consume_challenge(&request, "challenge-sqlite-02"));
        assert!(!reloaded.consume_challenge(&request, "challenge-sqlite-02"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn sqlite_activation_registry_keeps_both_entries_across_two_instances() {
        let _lock = lock_activation_registry_stats_for_test();
        let path = unique_test_file_path("activation-registry-sqlite-two-instances");
        let request_a = HttpRequest::new("POST", "/iot/xiaozhi/ota")
            .with_header("device-id", "device-a")
            .with_header("client-id", "client-a");
        let request_b = HttpRequest::new("POST", "/iot/xiaozhi/ota")
            .with_header("device-id", "device-b")
            .with_header("client-id", "client-b");

        let registry_a = SqliteXiaozhiActivationChallengeRegistry::new(path.clone());
        let registry_b = SqliteXiaozhiActivationChallengeRegistry::new(path.clone());

        registry_a.register_challenge(&request_a, "challenge-shared", 60_000);
        registry_b.register_challenge(&request_b, "challenge-shared", 60_000);

        assert!(registry_a.consume_challenge(&request_a, "challenge-shared"));
        assert!(registry_b.consume_challenge(&request_b, "challenge-shared"));
        assert!(!registry_a.consume_challenge(&request_a, "challenge-shared"));
        assert!(!registry_b.consume_challenge(&request_b, "challenge-shared"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn sqlite_activation_registry_rejects_expired_entries() {
        let _lock = lock_activation_registry_stats_for_test();
        let path = unique_test_file_path("activation-registry-sqlite-expired");
        let request = HttpRequest::new("POST", "/iot/xiaozhi/ota")
            .with_header("device-id", "device-sqlite-exp")
            .with_header("client-id", "client-sqlite-exp");

        let registry = SqliteXiaozhiActivationChallengeRegistry::new(path.clone());
        registry.register_challenge(&request, "challenge-expired", 0);
        assert!(!registry.consume_challenge(&request, "challenge-expired"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn integer_schema_error_reports_external_style_range_messages() {
        let schema = r#"{"type":"integer","minimum":0,"maximum":100}"#;
        assert_eq!(
            integer_value_schema_error(-1, schema),
            Some("Value is below minimum allowed: 0".to_string())
        );
        assert_eq!(
            integer_value_schema_error(101, schema),
            Some("Value exceeds maximum allowed: 100".to_string())
        );
        assert_eq!(integer_value_schema_error(100, schema), None);
    }

    #[test]
    fn parse_json_number_and_truncate_supports_external_integer_semantics() {
        assert_eq!(parse_json_number_value("99.9"), Some(99.9));
        assert_eq!(truncate_to_i64(99.9), 99);
        assert_eq!(truncate_to_i64(-3.7), -3);
    }

    #[test]
    fn parse_mcp_policy_rules_supports_compound_predicates() {
        let rules = parse_mcp_policy_rules(
            "deny|tool=self.reboot|transport=websocket;allow|tool=self.reboot|transport=websocket|device_prefix=lab-|arg_volume_gte=80|arg_mode_str_eq=night|arg_enabled_bool_eq=true",
        );
        assert_eq!(rules.len(), 2);
        assert!(matches!(rules[0].decision, McpPolicyDecision::Deny));
        assert_eq!(rules[0].pattern.tool.as_deref(), Some("self.reboot"));
        assert_eq!(rules[0].pattern.transport.as_deref(), Some("websocket"));
        assert_eq!(rules[1].pattern.device_prefix.as_deref(), Some("lab-"));
        assert_eq!(rules[1].pattern.numeric_arg_predicates.len(), 1);
        assert_eq!(rules[1].pattern.numeric_arg_predicates[0].field, "volume");
        assert!(matches!(
            rules[1].pattern.numeric_arg_predicates[0].operator,
            McpPolicyNumericOperator::Gte
        ));
        assert!((rules[1].pattern.numeric_arg_predicates[0].expected - 80.0).abs() < 1e-9);
        assert_eq!(rules[1].pattern.string_arg_predicates.len(), 1);
        assert_eq!(rules[1].pattern.string_arg_predicates[0].field, "mode");
        assert!(matches!(
            rules[1].pattern.string_arg_predicates[0].operator,
            McpPolicyStringOperator::Eq
        ));
        assert_eq!(rules[1].pattern.string_arg_predicates[0].expected, "night");
        assert_eq!(rules[1].pattern.boolean_arg_predicates.len(), 1);
        assert_eq!(rules[1].pattern.boolean_arg_predicates[0].field, "enabled");
        assert!(matches!(
            rules[1].pattern.boolean_arg_predicates[0].operator,
            McpPolicyBooleanOperator::Eq
        ));
        assert!(rules[1].pattern.boolean_arg_predicates[0].expected);
    }

    #[test]
    fn rule_based_mcp_tool_policy_uses_first_match() {
        let policy = RuleBasedXiaozhiSimulatorMcpToolPolicy::from_rules(parse_mcp_policy_rules(
            "allow|tool=self.reboot|transport=websocket|device_prefix=lab-;deny|tool=self.reboot|transport=websocket",
        ));
        let tool = XiaozhiSimulatorMcpToolSpec::new(
            "self.reboot",
            "Reboot",
            r#"{"type":"object","properties":{},"required":[]}"#,
        );
        let ctx_allow =
            XiaozhiMcpInvocationContext::new("websocket", "session-a").with_device_id("lab-001");
        let ctx_deny =
            XiaozhiMcpInvocationContext::new("websocket", "session-b").with_device_id("prod-001");

        assert!(policy.allow(&ctx_allow, &tool, Some("{}")).is_ok());
        assert!(policy.allow(&ctx_deny, &tool, Some("{}")).is_err());
    }

    #[test]
    fn numeric_argument_predicates_require_matching_argument_values() {
        let predicates = vec![McpPolicyNumericArgumentPredicate {
            field: "volume".to_string(),
            operator: McpPolicyNumericOperator::Gte,
            expected: 80.0,
        }];
        assert!(mcp_policy_numeric_predicates_match(
            &predicates,
            Some(r#"{"volume":80}"#)
        ));
        assert!(mcp_policy_numeric_predicates_match(
            &predicates,
            Some(r#"{"volume":99.5}"#)
        ));
        assert!(!mcp_policy_numeric_predicates_match(
            &predicates,
            Some(r#"{"volume":79.9}"#)
        ));
        assert!(!mcp_policy_numeric_predicates_match(
            &predicates,
            Some(r#"{"brightness":100}"#)
        ));
    }

    #[test]
    fn string_and_boolean_argument_predicates_require_matching_values() {
        let string_predicates = vec![
            McpPolicyStringArgumentPredicate {
                field: "mode".to_string(),
                operator: McpPolicyStringOperator::Eq,
                expected: "night".to_string(),
            },
            McpPolicyStringArgumentPredicate {
                field: "profile".to_string(),
                operator: McpPolicyStringOperator::Prefix,
                expected: "lab-".to_string(),
            },
        ];
        assert!(mcp_policy_string_predicates_match(
            &string_predicates,
            Some(r#"{"mode":"night","profile":"lab-alpha"}"#)
        ));
        assert!(!mcp_policy_string_predicates_match(
            &string_predicates,
            Some(r#"{"mode":"day","profile":"lab-alpha"}"#)
        ));
        assert!(!mcp_policy_string_predicates_match(
            &string_predicates,
            Some(r#"{"mode":"night","profile":"prod-alpha"}"#)
        ));

        let boolean_predicates = vec![
            McpPolicyBooleanArgumentPredicate {
                field: "enabled".to_string(),
                operator: McpPolicyBooleanOperator::Eq,
                expected: true,
            },
            McpPolicyBooleanArgumentPredicate {
                field: "muted".to_string(),
                operator: McpPolicyBooleanOperator::Ne,
                expected: true,
            },
        ];
        assert!(mcp_policy_boolean_predicates_match(
            &boolean_predicates,
            Some(r#"{"enabled":true,"muted":false}"#)
        ));
        assert!(!mcp_policy_boolean_predicates_match(
            &boolean_predicates,
            Some(r#"{"enabled":false,"muted":false}"#)
        ));
        assert!(!mcp_policy_boolean_predicates_match(
            &boolean_predicates,
            Some(r#"{"enabled":true,"muted":true}"#)
        ));
    }

    #[test]
    fn mcp_policy_decision_log_line_includes_core_fields() {
        let context = XiaozhiMcpInvocationContext::new("mqtt", "session-01")
            .with_device_id("dev-01")
            .with_client_id("client-01");
        let evaluation = XiaozhiMcpPolicyEvaluation::deny("blocked by policy", Some(3));
        let line = mcp_policy_decision_log_line(&context, "self.reboot", &evaluation, true);
        assert!(line.contains("mcp_policy_decision"));
        assert!(line.contains("decision=deny"));
        assert!(line.contains("rule_index=3"));
        assert!(line.contains("transport=mqtt"));
        assert!(line.contains("session_id=session-01"));
        assert!(line.contains("device_id=dev-01"));
        assert!(line.contains("client_id=client-01"));
        assert!(line.contains("tool=self.reboot"));
        assert!(line.contains("arguments_present=true"));
        assert!(line.contains("message=\"blocked by policy\""));
    }

    #[test]
    fn rule_based_mcp_policy_records_counters_for_allow_and_deny_paths() {
        let policy = RuleBasedXiaozhiSimulatorMcpToolPolicy::from_rules(parse_mcp_policy_rules(
            "deny|tool=self.reboot|transport=websocket;allow|tool=self.reboot|transport=mqtt",
        ));
        let tool = XiaozhiSimulatorMcpToolSpec::new(
            "self.reboot",
            "Reboot",
            r#"{"type":"object","properties":{},"required":[]}"#,
        );

        let deny_ctx = XiaozhiMcpInvocationContext::new("websocket", "session-deny");
        let allow_by_rule_ctx = XiaozhiMcpInvocationContext::new("mqtt", "session-allow-rule");
        let allow_no_rule_ctx =
            XiaozhiMcpInvocationContext::new("websocket", "session-allow-default");

        assert!(policy.allow(&deny_ctx, &tool, Some("{}")).is_err());
        assert!(policy.allow(&allow_by_rule_ctx, &tool, Some("{}")).is_ok());
        let other_tool = XiaozhiSimulatorMcpToolSpec::new(
            "self.get_device_status",
            "Status",
            r#"{"type":"object","properties":{},"required":[]}"#,
        );
        assert!(policy
            .allow(&allow_no_rule_ctx, &other_tool, Some("{}"))
            .is_ok());

        let snapshot = policy.stats_snapshot();
        assert_eq!(snapshot.deny_by_rule_matches, 1);
        assert_eq!(snapshot.allow_by_rule_matches, 1);
        assert_eq!(snapshot.allow_no_rule_matches, 1);
    }

    #[test]
    fn rule_based_mcp_policy_evaluate_reports_matched_rule_index() {
        let policy = RuleBasedXiaozhiSimulatorMcpToolPolicy::from_rules(parse_mcp_policy_rules(
            "allow|tool=self.reboot|transport=mqtt;deny|tool=self.reboot|transport=websocket",
        ));
        let tool = XiaozhiSimulatorMcpToolSpec::new(
            "self.reboot",
            "Reboot",
            r#"{"type":"object","properties":{},"required":[]}"#,
        );

        let allow_ctx = XiaozhiMcpInvocationContext::new("mqtt", "session-allow");
        let deny_ctx = XiaozhiMcpInvocationContext::new("websocket", "session-deny");
        let none_ctx = XiaozhiMcpInvocationContext::new("mqtt", "session-none");
        let none_tool = XiaozhiSimulatorMcpToolSpec::new(
            "self.get_device_status",
            "Status",
            r#"{"type":"object","properties":{},"required":[]}"#,
        );

        let allow_eval = policy.evaluate(&allow_ctx, &tool, Some("{}"));
        assert_eq!(allow_eval.decision, XiaozhiMcpPolicyDecision::Allow);
        assert_eq!(allow_eval.matched_rule_index, Some(0));
        assert!(allow_eval.error_message.is_none());

        let deny_eval = policy.evaluate(&deny_ctx, &tool, Some("{}"));
        assert_eq!(deny_eval.decision, XiaozhiMcpPolicyDecision::Deny);
        assert_eq!(deny_eval.matched_rule_index, Some(1));
        assert_eq!(
            deny_eval.error_message.as_deref(),
            Some("Tool not allowed by policy: self.reboot")
        );

        let none_eval = policy.evaluate(&none_ctx, &none_tool, Some("{}"));
        assert_eq!(none_eval.decision, XiaozhiMcpPolicyDecision::Allow);
        assert_eq!(none_eval.matched_rule_index, None);
        assert!(none_eval.error_message.is_none());
    }

    fn unique_test_file_path(prefix: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{}-{now}.tmp", std::process::id()))
    }

    fn reset_activation_registry_stats_for_test() {
        ACTIVATION_REGISTRY_STATS
            .backend_kind
            .store(ACTIVATION_REGISTRY_BACKEND_UNKNOWN, Ordering::Relaxed);
        ACTIVATION_REGISTRY_STATS
            .register_total
            .store(0, Ordering::Relaxed);
        ACTIVATION_REGISTRY_STATS
            .consume_total
            .store(0, Ordering::Relaxed);
        ACTIVATION_REGISTRY_STATS
            .consume_hits
            .store(0, Ordering::Relaxed);
        ACTIVATION_REGISTRY_STATS
            .consume_misses
            .store(0, Ordering::Relaxed);
        ACTIVATION_REGISTRY_STATS
            .pruned_entries
            .store(0, Ordering::Relaxed);
    }

    fn lock_activation_registry_stats_for_test() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    struct EnvGuard {
        values: Vec<(String, Option<String>)>,
    }

    impl EnvGuard {
        fn set_all(vars: &[(&str, Option<&str>)]) -> Self {
            let mut values = Vec::with_capacity(vars.len());
            for (name, value) in vars {
                let previous = std::env::var(name).ok();
                values.push(((*name).to_string(), previous));
                match value {
                    Some(value) => std::env::set_var(name, value),
                    None => std::env::remove_var(name),
                }
            }
            Self { values }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (name, value) in &self.values {
                if let Some(value) = value {
                    std::env::set_var(name, value);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }

    #[test]
    fn activation_registry_stats_snapshot_reflects_in_memory_register_and_consume() {
        let _lock = lock_activation_registry_stats_for_test();
        reset_activation_registry_stats_for_test();
        let registry = InMemoryXiaozhiActivationChallengeRegistry::new();
        let request = HttpRequest::new("POST", "/iot/xiaozhi/ota")
            .with_header("device-id", "stats-device-1")
            .with_header("client-id", "stats-client-1");

        registry.register_challenge(&request, "stats-challenge", 60_000);
        assert!(registry.consume_challenge(&request, "stats-challenge"));
        assert!(!registry.consume_challenge(&request, "stats-challenge"));

        let snapshot = xiaozhi_activation_registry_stats_snapshot();
        assert_eq!(snapshot.backend_kind, "in_memory");
        assert_eq!(snapshot.register_total, 1);
        assert_eq!(snapshot.consume_total, 2);
        assert_eq!(snapshot.consume_hits, 1);
        assert_eq!(snapshot.consume_misses, 1);
    }

    #[test]
    fn activation_registry_stats_snapshot_reflects_sqlite_backend_selection() {
        let _lock = lock_activation_registry_stats_for_test();
        reset_activation_registry_stats_for_test();
        let path = unique_test_file_path("activation-registry-stats-sqlite");
        let registry = SqliteXiaozhiActivationChallengeRegistry::new(path.clone());
        let request = HttpRequest::new("POST", "/iot/xiaozhi/ota")
            .with_header("device-id", "stats-device-2")
            .with_header("client-id", "stats-client-2");

        registry.register_challenge(&request, "stats-challenge", 60_000);
        let snapshot = xiaozhi_activation_registry_stats_snapshot();
        assert_eq!(snapshot.backend_kind, "sqlite");
        assert_eq!(snapshot.register_total, 1);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn activation_registry_stats_snapshot_reflects_redis_backend_name() {
        let _lock = lock_activation_registry_stats_for_test();
        reset_activation_registry_stats_for_test();
        activation_registry_mark_backend(ACTIVATION_REGISTRY_BACKEND_REDIS);
        let snapshot = xiaozhi_activation_registry_stats_snapshot();
        assert_eq!(snapshot.backend_kind, "redis");
    }

    #[test]
    fn activation_registry_from_env_falls_back_to_in_memory_when_redis_url_missing() {
        let _lock = lock_activation_registry_stats_for_test();
        reset_activation_registry_stats_for_test();
        let _guard = EnvGuard::set_all(&[
            (
                "SDKWORK_AIOT_XIAOZHI_ACTIVATION_REGISTRY_KIND",
                Some("redis"),
            ),
            ("SDKWORK_AIOT_XIAOZHI_ACTIVATION_REGISTRY_REDIS_URL", None),
            ("SDKWORK_AIOT_XIAOZHI_ACTIVATION_REGISTRY_PATH", None),
        ]);
        let registry = activation_challenge_registry_from_env();
        let request = HttpRequest::new("POST", "/iot/xiaozhi/ota")
            .with_header("device-id", "stats-device-fallback")
            .with_header("client-id", "stats-client-fallback");

        registry.register_challenge(&request, "stats-fallback", 60_000);
        assert!(registry.consume_challenge(&request, "stats-fallback"));

        let snapshot = xiaozhi_activation_registry_stats_snapshot();
        assert_eq!(snapshot.backend_kind, "in_memory");
        assert_eq!(snapshot.register_total, 1);
        assert_eq!(snapshot.consume_total, 1);
        assert_eq!(snapshot.consume_hits, 1);
        assert_eq!(snapshot.consume_misses, 0);
    }
}
