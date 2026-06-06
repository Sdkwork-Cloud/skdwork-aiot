use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::types::Value as SqliteValue;
use rusqlite::{params_from_iter, Connection};
use sdkwork_aiot_storage::{
    table_contract, AiotCommandCreateCommand, AiotCommandRecord, AiotCommandRepository,
    AiotCommandRepositoryError, AiotCommandResultRecord, AiotDeviceCreateCommand,
    AiotDeviceEventCreateCommand, AiotDeviceEventRecord, AiotDeviceRecord, AiotDeviceRepository,
    AiotDeviceRepositoryError, AiotDeviceSessionRepository, AiotDeviceTwinRepository,
    AiotDeviceTwinRepositoryError, AiotDeviceTwinSnapshot, AiotDeviceUpdateCommand,
    AiotEventRepository, AiotEventRepositoryError, AiotProtocolDeadLetterIntent,
    AiotProtocolIngestUnitOfWork, AiotProtocolStorageCommand, AiotStorageAssociation,
    AiotStorageWriteReceipt, AiotTwinPropertyUpsertCommand,
};
use serde_json::Value as JsonValue;

pub fn schema_version() -> &'static str {
    "0.2.0"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlMigration {
    pub version: &'static str,
    pub name: &'static str,
    pub schema_version: &'static str,
    pub sql: &'static str,
}

pub fn migration_catalog() -> Vec<SqlMigration> {
    vec![SqlMigration {
        version: "0001",
        name: "aiot_core_schema",
        schema_version: schema_version(),
        sql: initial_migration_sql(),
    }]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlDeviceWriteOperation {
    Create(AiotDeviceRecord),
    Update(AiotDeviceRecord),
    Delete {
        association: AiotStorageAssociation,
        device_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SqlDeviceRepositoryPlanner {
    dialect: SqlDialect,
}

impl SqlDeviceRepositoryPlanner {
    pub fn standard() -> Self {
        Self {
            dialect: SqlDialect::Postgres,
        }
    }

    pub fn with_dialect(dialect: SqlDialect) -> Self {
        Self { dialect }
    }

    pub fn plan_create_device(
        &self,
        device: &AiotDeviceRecord,
    ) -> Result<SqlStatementBatch, SqlPlanError> {
        let batch = SqlStatementBatch::single(
            "device_create",
            device_create_statement(self.dialect, device)?,
        );
        batch.validate()?;
        Ok(batch)
    }

    pub fn plan_update_device(
        &self,
        device: &AiotDeviceRecord,
    ) -> Result<SqlStatementBatch, SqlPlanError> {
        let batch = SqlStatementBatch::single(
            "device_update",
            device_update_statement(self.dialect, device)?,
        );
        batch.validate()?;
        Ok(batch)
    }

    pub fn plan_delete_device(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
    ) -> Result<SqlStatementBatch, SqlPlanError> {
        let batch = SqlStatementBatch::single(
            "device_delete",
            device_delete_statement(self.dialect, association, device_id),
        );
        batch.validate()?;
        Ok(batch)
    }
}

impl Default for SqlDeviceRepositoryPlanner {
    fn default() -> Self {
        Self::standard()
    }
}

#[derive(Debug, Default)]
struct InMemorySqlxDeviceRepositoryState {
    next_device_pk: u64,
    devices: BTreeMap<String, AiotDeviceRecord>,
    next_command_pk: u64,
    commands: BTreeMap<String, AiotCommandRecord>,
    command_idempotency_index: BTreeMap<String, String>,
    next_event_pk: u64,
    events: Vec<AiotDeviceEventRecord>,
    twins: BTreeMap<String, AiotDeviceTwinSnapshot>,
    disconnected_sessions: BTreeSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct InMemorySqlxDeviceRepository {
    executor: InMemorySqlStatementExecutor,
    planner: SqlDeviceRepositoryPlanner,
    state: Arc<Mutex<InMemorySqlxDeviceRepositoryState>>,
    writes: Arc<Mutex<Vec<SqlDeviceWriteOperation>>>,
}

impl InMemorySqlxDeviceRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn writes(&self) -> Vec<SqlDeviceWriteOperation> {
        self.writes
            .lock()
            .expect("sqlx device repo writes poisoned")
            .clone()
    }

    pub fn executed_statements(&self) -> Vec<SqlStatementPlan> {
        self.executor.executed_statements()
    }
}

impl AiotDeviceRepository for InMemorySqlxDeviceRepository {
    fn create_device(
        &self,
        command: AiotDeviceCreateCommand,
    ) -> Result<AiotDeviceRecord, AiotDeviceRepositoryError> {
        if !is_valid_int64_string(&command.product_id) {
            return Err(AiotDeviceRepositoryError::InvalidProductId);
        }

        let mut state = self.state.lock().expect("sqlx device repo state poisoned");
        let key = scoped_device_key(&command.association, &command.device_id);
        if state.devices.contains_key(&key) {
            return Err(AiotDeviceRepositoryError::DuplicateDeviceId);
        }

        let record = AiotDeviceRecord {
            id: (state.next_device_pk + 1).to_string(),
            tenant_id: command.association.tenant_id,
            organization_id: command.association.organization_id,
            device_id: command.device_id,
            display_name: command.display_name,
            product_id: command.product_id,
            client_id: command.client_id,
            chip_family: command.chip_family,
            status: "active".to_string(),
            metadata_json: None,
            last_seen_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let batch = self
            .planner
            .plan_create_device(&record)
            .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
        self.executor.execute_batch(batch);
        state.next_device_pk += 1;
        state.devices.insert(key, record.clone());
        self.writes
            .lock()
            .expect("sqlx device repo writes poisoned")
            .push(SqlDeviceWriteOperation::Create(record.clone()));
        Ok(record)
    }

    fn get_device(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
    ) -> Option<AiotDeviceRecord> {
        self.state
            .lock()
            .expect("sqlx device repo state poisoned")
            .devices
            .get(&scoped_device_key(association, device_id))
            .cloned()
    }

    fn list_devices(&self, association: &AiotStorageAssociation) -> Vec<AiotDeviceRecord> {
        self.state
            .lock()
            .expect("sqlx device repo state poisoned")
            .devices
            .values()
            .filter(|device| {
                device.tenant_id == association.tenant_id
                    && device.organization_id == association.organization_id
            })
            .cloned()
            .collect()
    }

    fn update_device(
        &self,
        command: AiotDeviceUpdateCommand,
    ) -> Result<AiotDeviceRecord, AiotDeviceRepositoryError> {
        let mut state = self.state.lock().expect("sqlx device repo state poisoned");
        let key = scoped_device_key(&command.association, &command.device_id);
        let Some(device) = state.devices.get_mut(&key) else {
            return Err(AiotDeviceRepositoryError::NotFound);
        };
        if let Some(display_name) = command.display_name {
            device.display_name = display_name;
        }
        if let Some(status) = command.status {
            device.status = status;
        }
        if command.metadata_json.is_some() {
            device.metadata_json = command.metadata_json;
        }
        let record = device.clone();
        let batch = self
            .planner
            .plan_update_device(&record)
            .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
        self.executor.execute_batch(batch);
        self.writes
            .lock()
            .expect("sqlx device repo writes poisoned")
            .push(SqlDeviceWriteOperation::Update(record.clone()));
        Ok(record)
    }

    fn delete_device(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
    ) -> Result<(), AiotDeviceRepositoryError> {
        let mut state = self.state.lock().expect("sqlx device repo state poisoned");
        let key = scoped_device_key(association, device_id);
        if state.devices.remove(&key).is_none() {
            return Err(AiotDeviceRepositoryError::NotFound);
        }
        let batch = self
            .planner
            .plan_delete_device(association, device_id)
            .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
        self.executor.execute_batch(batch);
        self.writes
            .lock()
            .expect("sqlx device repo writes poisoned")
            .push(SqlDeviceWriteOperation::Delete {
                association: association.clone(),
                device_id: device_id.to_string(),
            });
        Ok(())
    }
}

impl AiotCommandRepository for InMemorySqlxDeviceRepository {
    fn create_command(
        &self,
        command: AiotCommandCreateCommand,
    ) -> Result<AiotCommandRecord, AiotCommandRepositoryError> {
        let mut state = self.state.lock().expect("sqlx device repo state poisoned");
        if let Some(idempotency_key) = command.idempotency_key.as_deref() {
            let idempotency_scope_key = format!(
                "{}:{}:{idempotency_key}",
                command.association.tenant_id, command.association.organization_id
            );
            if let Some(existing_command_key) =
                state.command_idempotency_index.get(&idempotency_scope_key)
            {
                if let Some(existing) = state.commands.get(existing_command_key) {
                    return Ok(existing.clone());
                }
            }
        }

        let command_id = command.command_id.unwrap_or_else(|| {
            format!(
                "cmd-{}-{:04}",
                command.device_id,
                state.next_command_pk.saturating_add(1)
            )
        });
        let command_key = scoped_command_key(&command.association, &command_id);
        if state.commands.contains_key(&command_key) {
            return Err(AiotCommandRepositoryError::DuplicateCommandId);
        }
        let idempotency_key = command.idempotency_key.clone();

        let record = AiotCommandRecord {
            id: state.next_command_pk.saturating_add(1).to_string(),
            tenant_id: command.association.tenant_id,
            organization_id: command.association.organization_id,
            command_id,
            device_id: command.device_id,
            session_id: command.session_id,
            capability_name: command.capability_name,
            command_name: command.command_name,
            request_payload_json: command.request_payload_json,
            request_media_resource_id: command.request_media_resource_id,
            request_object_blob_id: command.request_object_blob_id,
            request_media_json: command.request_media_json,
            status: command.status,
            trace_id: command.trace_id,
            timeout_at: command.timeout_at,
            ack_at: None,
            result_at: None,
            created_at: default_timestamp().to_string(),
            result: None,
        };
        state.next_command_pk = state.next_command_pk.saturating_add(1);
        state.commands.insert(command_key, record.clone());
        if let Some(idempotency_key) = idempotency_key {
            let idempotency_scope_key = format!(
                "{}:{}:{idempotency_key}",
                command.association.tenant_id, command.association.organization_id
            );
            let command_key = scoped_command_key(&command.association, &record.command_id);
            state
                .command_idempotency_index
                .insert(idempotency_scope_key, command_key);
        }
        Ok(record)
    }

    fn list_commands(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
    ) -> Result<Vec<AiotCommandRecord>, AiotCommandRepositoryError> {
        let mut commands = self
            .state
            .lock()
            .expect("sqlx device repo state poisoned")
            .commands
            .values()
            .filter(|command| {
                command.tenant_id == association.tenant_id
                    && command.organization_id == association.organization_id
                    && command.device_id == device_id
            })
            .cloned()
            .collect::<Vec<_>>();
        commands.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(commands)
    }

    fn cancel_command(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
        command_id: &str,
    ) -> Result<Option<AiotCommandRecord>, AiotCommandRepositoryError> {
        let mut state = self.state.lock().expect("sqlx device repo state poisoned");
        let key = scoped_command_key(association, command_id);
        let Some(command) = state.commands.get_mut(&key) else {
            return Ok(None);
        };
        if command.device_id != device_id {
            return Ok(None);
        }
        command.status = "cancelled".to_string();
        Ok(Some(command.clone()))
    }
}

impl AiotDeviceSessionRepository for InMemorySqlxDeviceRepository {
    fn disconnect_session(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
        session_id: &str,
    ) -> Result<bool, AiotDeviceRepositoryError> {
        let mut state = self.state.lock().expect("sqlx device repo state poisoned");
        let key = scoped_device_session_key(association, device_id, session_id);
        Ok(state.disconnected_sessions.insert(key))
    }

    fn is_session_disconnected(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
        session_id: &str,
    ) -> Result<bool, AiotDeviceRepositoryError> {
        let state = self.state.lock().expect("sqlx device repo state poisoned");
        Ok(state
            .disconnected_sessions
            .contains(&scoped_device_session_key(
                association,
                device_id,
                session_id,
            )))
    }
}

impl AiotEventRepository for InMemorySqlxDeviceRepository {
    fn record_event(
        &self,
        command: AiotDeviceEventCreateCommand,
    ) -> Result<AiotDeviceEventRecord, AiotEventRepositoryError> {
        let mut state = self.state.lock().expect("sqlx device repo state poisoned");
        let next_event_pk = state.next_event_pk.saturating_add(1);
        let event_id = command
            .event_id
            .unwrap_or_else(|| format!("evt-{}-{:04}", command.device_id, next_event_pk));
        let event = AiotDeviceEventRecord {
            id: next_event_pk.to_string(),
            tenant_id: command.association.tenant_id,
            organization_id: command.association.organization_id,
            event_id,
            event_type: command.event_type,
            event_version: command.event_version,
            device_id: command.device_id,
            protocol_id: command.protocol_id,
            adapter_id: command.adapter_id,
            message_class: command.message_class,
            semantic_type: command.semantic_type,
            transport: command.transport,
            direction: command.direction,
            message_id: command.message_id,
            correlation_id: command.correlation_id,
            trace_id: command.trace_id,
            payload_hash: command.payload_hash,
            media_resource_id: command.media_resource_id,
            object_blob_id: command.object_blob_id,
            media_json: command.media_json,
            payload_json: command.payload_json,
            occurred_at: command.occurred_at,
        };
        state.next_event_pk = next_event_pk;
        state.events.push(event.clone());
        Ok(event)
    }

    fn list_events(
        &self,
        association: &AiotStorageAssociation,
        device_id: Option<&str>,
    ) -> Result<Vec<AiotDeviceEventRecord>, AiotEventRepositoryError> {
        let mut events = self
            .state
            .lock()
            .expect("sqlx device repo state poisoned")
            .events
            .iter()
            .filter(|event| {
                event.tenant_id == association.tenant_id
                    && event.organization_id == association.organization_id
                    && device_id
                        .map(|scoped_device_id| scoped_device_id == event.device_id)
                        .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();
        events.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(events)
    }
}

impl AiotDeviceTwinRepository for InMemorySqlxDeviceRepository {
    fn upsert_twin_property(
        &self,
        command: AiotTwinPropertyUpsertCommand,
    ) -> Result<AiotDeviceTwinSnapshot, AiotDeviceTwinRepositoryError> {
        let mut state = self.state.lock().expect("sqlx device repo state poisoned");
        let twin_key = scoped_device_key(&command.association, &command.device_id);
        let snapshot = state
            .twins
            .entry(twin_key)
            .or_insert_with(|| empty_twin_snapshot(&command.association, &command.device_id));
        if let Some(desired) = command.desired_value_json {
            snapshot
                .desired
                .insert(command.property_name.clone(), desired);
            snapshot.desired_version = snapshot.desired_version.saturating_add(1);
        }
        if let Some(reported) = command.reported_value_json {
            snapshot
                .reported
                .insert(command.property_name.clone(), reported);
            snapshot.reported_version = snapshot.reported_version.saturating_add(1);
        }
        snapshot.updated_at = command
            .desired_updated_at
            .or(command.reported_updated_at)
            .unwrap_or_else(|| default_timestamp().to_string());
        Ok(snapshot.clone())
    }

    fn get_twin_snapshot(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
    ) -> Result<AiotDeviceTwinSnapshot, AiotDeviceTwinRepositoryError> {
        let state = self.state.lock().expect("sqlx device repo state poisoned");
        let twin_key = scoped_device_key(association, device_id);
        Ok(state
            .twins
            .get(&twin_key)
            .cloned()
            .unwrap_or_else(|| empty_twin_snapshot(association, device_id)))
    }
}

fn scoped_device_key(association: &AiotStorageAssociation, device_id: &str) -> String {
    format!(
        "{}:{}:{}",
        association.tenant_id, association.organization_id, device_id
    )
}

fn is_valid_int64_string(value: &str) -> bool {
    if value.is_empty() || !value.as_bytes().iter().all(u8::is_ascii_digit) {
        return false;
    }

    value.parse::<i64>().is_ok()
}

#[derive(Debug, Clone)]
pub struct SqliteSqlxDeviceRepository {
    connection: Arc<Mutex<Connection>>,
    planner: SqlDeviceRepositoryPlanner,
    command_idempotency_cache: Arc<Mutex<HashMap<(i64, i64, String), String>>>,
}

impl SqliteSqlxDeviceRepository {
    pub fn new_in_memory() -> Result<Self, rusqlite::Error> {
        let connection = Connection::open_in_memory()?;
        ensure_device_schema(&connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            planner: SqlDeviceRepositoryPlanner::with_dialect(SqlDialect::Sqlite),
            command_idempotency_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, rusqlite::Error> {
        let connection = Connection::open(path)?;
        ensure_device_schema(&connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            planner: SqlDeviceRepositoryPlanner::with_dialect(SqlDialect::Sqlite),
            command_idempotency_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn execute_batch(&self, batch: SqlStatementBatch) -> Result<(), rusqlite::Error> {
        let mut connection = self.connection.lock().expect("sqlite device repo poisoned");
        let tx = connection.transaction()?;
        for statement in batch.statements {
            execute_sql_statement(&tx, &statement)?;
        }
        tx.commit()?;
        Ok(())
    }
}

impl AiotDeviceRepository for SqliteSqlxDeviceRepository {
    fn create_device(
        &self,
        command: AiotDeviceCreateCommand,
    ) -> Result<AiotDeviceRecord, AiotDeviceRepositoryError> {
        if !is_valid_int64_string(&command.product_id) {
            return Err(AiotDeviceRepositoryError::InvalidProductId);
        }

        if self
            .get_device(&command.association, &command.device_id)
            .is_some()
        {
            return Err(AiotDeviceRepositoryError::DuplicateDeviceId);
        }

        let next_id = {
            let connection = self.connection.lock().expect("sqlite device repo poisoned");
            let max_id: i64 = connection
                .query_row("SELECT COALESCE(MAX(id), 0) FROM iot_device", [], |row| {
                    row.get(0)
                })
                .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
            max_id + 1
        };

        let record = AiotDeviceRecord {
            id: next_id.to_string(),
            tenant_id: command.association.tenant_id,
            organization_id: command.association.organization_id,
            device_id: command.device_id,
            display_name: command.display_name,
            product_id: command.product_id,
            client_id: command.client_id,
            chip_family: command.chip_family,
            status: "active".to_string(),
            metadata_json: None,
            last_seen_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let batch = self
            .planner
            .plan_create_device(&record)
            .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
        self.execute_batch(batch)
            .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
        Ok(record)
    }

    fn get_device(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
    ) -> Option<AiotDeviceRecord> {
        let connection = self.connection.lock().expect("sqlite device repo poisoned");
        let mut statement = connection
            .prepare(
                "SELECT id, tenant_id, organization_id, device_id, display_name, product_id, client_id, chip_family, status, metadata, last_seen_at FROM iot_device WHERE tenant_id = ?1 AND organization_id = ?2 AND device_id = ?3 LIMIT 1",
            )
            .ok()?;
        statement
            .query_row(
                (
                    association.tenant_id,
                    association.organization_id,
                    device_id,
                ),
                |row| {
                    let id: i64 = row.get(0)?;
                    let tenant_id: i64 = row.get(1)?;
                    let organization_id: i64 = row.get(2)?;
                    let device_id: String = row.get(3)?;
                    let display_name: String = row.get(4)?;
                    let product_id: i64 = row.get(5)?;
                    let client_id: Option<String> = row.get(6)?;
                    let chip_family: Option<String> = row.get(7)?;
                    let status: i64 = row.get(8)?;
                    let metadata_json: Option<String> = row.get(9)?;
                    let last_seen_at: Option<String> = row.get(10)?;
                    Ok(AiotDeviceRecord {
                        id: id.to_string(),
                        tenant_id,
                        organization_id,
                        device_id,
                        display_name,
                        product_id: product_id.to_string(),
                        client_id,
                        chip_family,
                        status: device_status_text(status),
                        metadata_json,
                        last_seen_at: last_seen_at
                            .unwrap_or_else(|| "2026-01-01T00:00:00Z".to_string()),
                    })
                },
            )
            .ok()
    }

    fn list_devices(&self, association: &AiotStorageAssociation) -> Vec<AiotDeviceRecord> {
        let connection = self.connection.lock().expect("sqlite device repo poisoned");
        let mut statement = match connection.prepare(
            "SELECT id, tenant_id, organization_id, device_id, display_name, product_id, client_id, chip_family, status, metadata, last_seen_at FROM iot_device WHERE tenant_id = ?1 AND organization_id = ?2 ORDER BY id ASC",
        ) {
            Ok(statement) => statement,
            Err(_) => return Vec::new(),
        };
        let rows = match statement.query_map(
            (association.tenant_id, association.organization_id),
            |row| {
                let id: i64 = row.get(0)?;
                let tenant_id: i64 = row.get(1)?;
                let organization_id: i64 = row.get(2)?;
                let device_id: String = row.get(3)?;
                let display_name: String = row.get(4)?;
                let product_id: i64 = row.get(5)?;
                let client_id: Option<String> = row.get(6)?;
                let chip_family: Option<String> = row.get(7)?;
                let status: i64 = row.get(8)?;
                let metadata_json: Option<String> = row.get(9)?;
                let last_seen_at: Option<String> = row.get(10)?;
                Ok(AiotDeviceRecord {
                    id: id.to_string(),
                    tenant_id,
                    organization_id,
                    device_id,
                    display_name,
                    product_id: product_id.to_string(),
                    client_id,
                    chip_family,
                    status: device_status_text(status),
                    metadata_json,
                    last_seen_at: last_seen_at
                        .unwrap_or_else(|| "2026-01-01T00:00:00Z".to_string()),
                })
            },
        ) {
            Ok(rows) => rows,
            Err(_) => return Vec::new(),
        };
        rows.filter_map(Result::ok).collect()
    }

    fn update_device(
        &self,
        command: AiotDeviceUpdateCommand,
    ) -> Result<AiotDeviceRecord, AiotDeviceRepositoryError> {
        let Some(mut existing) = self.get_device(&command.association, &command.device_id) else {
            return Err(AiotDeviceRepositoryError::NotFound);
        };
        if let Some(display_name) = command.display_name {
            existing.display_name = display_name;
        }
        if let Some(status) = command.status {
            existing.status = status;
        }
        if command.metadata_json.is_some() {
            existing.metadata_json = command.metadata_json;
        }
        let batch = self
            .planner
            .plan_update_device(&existing)
            .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
        self.execute_batch(batch)
            .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
        Ok(existing)
    }

    fn delete_device(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
    ) -> Result<(), AiotDeviceRepositoryError> {
        if self.get_device(association, device_id).is_none() {
            return Err(AiotDeviceRepositoryError::NotFound);
        }
        let batch = self
            .planner
            .plan_delete_device(association, device_id)
            .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
        self.execute_batch(batch)
            .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
        Ok(())
    }
}

impl AiotCommandRepository for SqliteSqlxDeviceRepository {
    fn create_command(
        &self,
        command: AiotCommandCreateCommand,
    ) -> Result<AiotCommandRecord, AiotCommandRepositoryError> {
        if let Some(idempotency_key) = command.idempotency_key.as_deref() {
            let cache_key = (
                command.association.tenant_id,
                command.association.organization_id,
                idempotency_key.to_string(),
            );
            if let Some(existing_command_id) = self
                .command_idempotency_cache
                .lock()
                .expect("sqlite command idempotency cache poisoned")
                .get(&cache_key)
                .cloned()
            {
                let existing = self
                    .list_commands(&command.association, &command.device_id)?
                    .into_iter()
                    .find(|record| record.command_id == existing_command_id);
                if let Some(existing) = existing {
                    return Ok(existing);
                }
            }
        }

        let mut connection = self.connection.lock().expect("sqlite device repo poisoned");
        let tx = connection
            .transaction()
            .map_err(|_| AiotCommandRepositoryError::PersistenceFailure)?;

        let next_id: i64 = tx
            .query_row("SELECT COALESCE(MAX(id), 0) FROM iot_command", [], |row| {
                row.get(0)
            })
            .map_err(|_| AiotCommandRepositoryError::PersistenceFailure)?;

        let command_id = command
            .command_id
            .unwrap_or_else(|| format!("cmd-{}-{:04}", command.device_id, next_id + 1));

        let duplicate_count: i64 = tx
            .query_row(
                "SELECT COUNT(1) FROM iot_command WHERE tenant_id = ?1 AND command_id = ?2",
                (command.association.tenant_id, command_id.as_str()),
                |row| row.get(0),
            )
            .map_err(|_| AiotCommandRepositoryError::PersistenceFailure)?;
        if duplicate_count > 0 {
            return Err(AiotCommandRepositoryError::DuplicateCommandId);
        }

        let request_media_snapshot = command.request_media_json.clone();
        let status_code = command_status_code(&command.status);
        let created_at = default_timestamp().to_string();
        let trace_id = command.trace_id.clone();
        let idempotency_key = command.idempotency_key.clone();
        let command_id_for_cache = command_id.clone();

        tx.execute(
            "INSERT INTO iot_command (id, uuid, tenant_id, organization_id, data_scope, command_id, device_id, session_id, capability_name, command_name, request_payload, request_media_resource_id, request_object_blob_id, request_media_resource_snapshot, status, idempotency_key, timeout_at, ack_at, result_at, trace_id, created_at, updated_at, version, created_by, updated_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, NULL, NULL, ?18, ?19, ?20, 0, ?21, ?22)",
            rusqlite::params![
                next_id + 1,
                format!("cmd-uuid-{}", next_id + 1),
                command.association.tenant_id,
                command.association.organization_id,
                command.association.data_scope as i64,
                command_id.as_str(),
                command.device_id.as_str(),
                command.session_id.as_deref(),
                command.capability_name.as_str(),
                command.command_name.as_str(),
                command.request_payload_json.as_str(),
                command.request_media_resource_id.as_deref(),
                command.request_object_blob_id.as_deref(),
                request_media_snapshot.as_deref(),
                status_code,
                command.idempotency_key.as_deref(),
                command.timeout_at.as_deref(),
                trace_id.as_deref(),
                created_at.as_str(),
                created_at.as_str(),
                command.association.created_by,
                command.association.updated_by,
            ],
        )
        .map_err(|_| AiotCommandRepositoryError::PersistenceFailure)?;

        tx.commit()
            .map_err(|_| AiotCommandRepositoryError::PersistenceFailure)?;

        if let Some(idempotency_key) = idempotency_key {
            self.command_idempotency_cache
                .lock()
                .expect("sqlite command idempotency cache poisoned")
                .insert(
                    (
                        command.association.tenant_id,
                        command.association.organization_id,
                        idempotency_key,
                    ),
                    command_id_for_cache.clone(),
                );
        }

        Ok(AiotCommandRecord {
            id: (next_id + 1).to_string(),
            tenant_id: command.association.tenant_id,
            organization_id: command.association.organization_id,
            command_id,
            device_id: command.device_id,
            session_id: command.session_id,
            capability_name: command.capability_name,
            command_name: command.command_name,
            request_payload_json: command.request_payload_json,
            request_media_resource_id: command.request_media_resource_id,
            request_object_blob_id: command.request_object_blob_id,
            request_media_json: request_media_snapshot,
            status: command.status,
            trace_id: command.trace_id,
            timeout_at: command.timeout_at,
            ack_at: None,
            result_at: None,
            created_at,
            result: None,
        })
    }

    fn list_commands(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
    ) -> Result<Vec<AiotCommandRecord>, AiotCommandRepositoryError> {
        let connection = self.connection.lock().expect("sqlite device repo poisoned");
        let mut statement = connection
            .prepare(
                "SELECT id, command_id, device_id, session_id, capability_name, command_name, request_payload, request_media_resource_id, request_object_blob_id, request_media_resource_snapshot, status, timeout_at, ack_at, result_at, trace_id, created_at FROM iot_command WHERE tenant_id = ?1 AND organization_id = ?2 AND device_id = ?3 ORDER BY id ASC",
            )
            .map_err(|_| AiotCommandRepositoryError::PersistenceFailure)?;
        let rows = statement
            .query_map(
                (
                    association.tenant_id,
                    association.organization_id,
                    device_id.to_string(),
                ),
                |row| {
                    let id: i64 = row.get(0)?;
                    let command_id: String = row.get(1)?;
                    let device_id: String = row.get(2)?;
                    let session_id: Option<String> = row.get(3)?;
                    let capability_name: String = row.get(4)?;
                    let command_name: String = row.get(5)?;
                    let request_payload_json: String = row.get(6)?;
                    let request_media_resource_id: Option<String> = row.get(7)?;
                    let request_object_blob_id: Option<String> = row.get(8)?;
                    let request_media_json: Option<String> = row.get(9)?;
                    let status_code: i64 = row.get(10)?;
                    let timeout_at: Option<String> = row.get(11)?;
                    let ack_at: Option<String> = row.get(12)?;
                    let result_at: Option<String> = row.get(13)?;
                    let trace_id: Option<String> = row.get(14)?;
                    let created_at: Option<String> = row.get(15)?;
                    Ok(AiotCommandRecord {
                        id: id.to_string(),
                        tenant_id: association.tenant_id,
                        organization_id: association.organization_id,
                        command_id: command_id.clone(),
                        device_id,
                        session_id,
                        capability_name,
                        command_name,
                        request_payload_json,
                        request_media_resource_id,
                        request_object_blob_id,
                        request_media_json,
                        status: command_status_text(status_code),
                        trace_id,
                        timeout_at,
                        ack_at,
                        result_at,
                        created_at: created_at.unwrap_or_else(|| default_timestamp().to_string()),
                        result: None,
                    })
                },
            )
            .map_err(|_| AiotCommandRepositoryError::PersistenceFailure)?;
        let mut commands = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| AiotCommandRepositoryError::PersistenceFailure)?;

        for command in &mut commands {
            command.result = command_result_for(
                &connection,
                association.tenant_id,
                association.organization_id,
                &command.command_id,
            )?;
        }
        Ok(commands)
    }

    fn cancel_command(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
        command_id: &str,
    ) -> Result<Option<AiotCommandRecord>, AiotCommandRepositoryError> {
        {
            let mut connection = self.connection.lock().expect("sqlite device repo poisoned");
            let tx = connection
                .transaction()
                .map_err(|_| AiotCommandRepositoryError::PersistenceFailure)?;

            let existing = tx
                .query_row(
                    "SELECT id, status FROM iot_command WHERE tenant_id = ?1 AND organization_id = ?2 AND device_id = ?3 AND command_id = ?4 LIMIT 1",
                    (
                        association.tenant_id,
                        association.organization_id,
                        device_id,
                        command_id,
                    ),
                    |row| {
                        let id: i64 = row.get(0)?;
                        let status: i64 = row.get(1)?;
                        Ok((id, status))
                    },
                )
                .ok();

            let Some((id, current_status_code)) = existing else {
                return Ok(None);
            };

            if current_status_code != command_status_code("cancelled") {
                let now = default_timestamp().to_string();
                tx.execute(
                    "UPDATE iot_command SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    (command_status_code("cancelled"), now.as_str(), id),
                )
                .map_err(|_| AiotCommandRepositoryError::PersistenceFailure)?;
            }

            tx.commit()
                .map_err(|_| AiotCommandRepositoryError::PersistenceFailure)?;
        }

        let command = self
            .list_commands(association, device_id)?
            .into_iter()
            .find(|record| record.command_id == command_id);
        Ok(command)
    }
}

impl AiotDeviceSessionRepository for SqliteSqlxDeviceRepository {
    fn disconnect_session(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
        session_id: &str,
    ) -> Result<bool, AiotDeviceRepositoryError> {
        let mut connection = self.connection.lock().expect("sqlite device repo poisoned");
        let tx = connection
            .transaction()
            .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
        let now = default_timestamp().to_string();
        let disconnected_status = 2_i64;

        let existing_status = tx
            .query_row(
                "SELECT status FROM iot_device_session WHERE tenant_id = ?1 AND organization_id = ?2 AND device_id = ?3 AND session_id = ?4 LIMIT 1",
                (
                    association.tenant_id,
                    association.organization_id,
                    device_id,
                    session_id,
                ),
                |row| row.get::<_, i64>(0),
            )
            .ok();
        match existing_status {
            Some(status) if status == disconnected_status => {
                return Ok(false);
            }
            Some(_) => {
                tx.execute(
                    "UPDATE iot_device_session SET status = ?1, disconnected_at = ?2, updated_at = ?2 WHERE tenant_id = ?3 AND organization_id = ?4 AND device_id = ?5 AND session_id = ?6",
                    (
                        disconnected_status,
                        now.as_str(),
                        association.tenant_id,
                        association.organization_id,
                        device_id,
                        session_id,
                    ),
                )
                .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
            }
            None => {
                let next_id: i64 = tx
                    .query_row(
                        "SELECT COALESCE(MAX(id), 0) FROM iot_device_session",
                        [],
                        |row| row.get(0),
                    )
                    .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
                let session_uuid = format!("session-{session_id}");
                let connection_id = format!("connection-{session_id}");
                tx.execute(
                    "INSERT INTO iot_device_session (id, uuid, tenant_id, organization_id, data_scope, device_id, session_id, connection_id, protocol_id, adapter_id, node_id, status, connected_at, last_seen_at, disconnected_at, created_at, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'xiaozhi.websocket', 'xiaozhi', NULL, ?9, ?10, ?10, ?10, ?10, ?10, 0)",
                    rusqlite::params![
                        next_id + 1,
                        session_uuid.as_str(),
                        association.tenant_id,
                        association.organization_id,
                        association.data_scope as i64,
                        device_id,
                        session_id,
                        connection_id.as_str(),
                        disconnected_status,
                        now.as_str(),
                    ],
                )
                .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;
            }
        }

        tx.commit()
            .map_err(|_| AiotDeviceRepositoryError::PersistenceFailure)?;

        Ok(true)
    }

    fn is_session_disconnected(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
        session_id: &str,
    ) -> Result<bool, AiotDeviceRepositoryError> {
        let connection = self.connection.lock().expect("sqlite device repo poisoned");
        let disconnected_status = 2_i64;
        let status = connection
            .query_row(
                "SELECT status FROM iot_device_session WHERE tenant_id = ?1 AND organization_id = ?2 AND device_id = ?3 AND session_id = ?4 LIMIT 1",
                (
                    association.tenant_id,
                    association.organization_id,
                    device_id,
                    session_id,
                ),
                |row| row.get::<_, i64>(0),
            )
            .ok();
        Ok(status == Some(disconnected_status))
    }
}

impl AiotEventRepository for SqliteSqlxDeviceRepository {
    fn record_event(
        &self,
        command: AiotDeviceEventCreateCommand,
    ) -> Result<AiotDeviceEventRecord, AiotEventRepositoryError> {
        let mut connection = self.connection.lock().expect("sqlite device repo poisoned");
        let tx = connection
            .transaction()
            .map_err(|_| AiotEventRepositoryError::PersistenceFailure)?;

        let next_id: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(id), 0) FROM iot_device_event",
                [],
                |row| row.get(0),
            )
            .map_err(|_| AiotEventRepositoryError::PersistenceFailure)?;

        let event_id = command
            .event_id
            .unwrap_or_else(|| format!("evt-{}-{:04}", command.device_id, next_id + 1));
        let media_snapshot = command.media_json.clone();
        let occurred_at = command.occurred_at.clone();
        let envelope_payload = serde_json::json!({
            "eventVersion": command.event_version,
            "protocolId": command.protocol_id,
            "adapterId": command.adapter_id,
            "messageClass": command.message_class,
            "semanticType": command.semantic_type,
            "transport": command.transport,
            "direction": command.direction,
            "messageId": command.message_id,
            "correlationId": command.correlation_id,
            "traceId": command.trace_id,
            "payloadHash": command.payload_hash,
            "occurredAt": occurred_at,
            "payload": serde_json::from_str::<JsonValue>(&command.payload_json).unwrap_or_else(|_| JsonValue::String(command.payload_json.clone()))
        });
        let event_payload_json = envelope_payload.to_string();

        tx.execute(
            "INSERT INTO iot_device_event (id, uuid, tenant_id, organization_id, data_scope, device_id, event_type, event_payload, media_resource_id, object_blob_id, media_resource_snapshot, status, created_at, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1, ?12, ?13, 0)",
            (
                next_id + 1,
                event_id.as_str(),
                command.association.tenant_id,
                command.association.organization_id,
                command.association.data_scope as i64,
                command.device_id.as_str(),
                command.event_type.as_str(),
                event_payload_json.as_str(),
                command.media_resource_id.as_deref(),
                command.object_blob_id.as_deref(),
                media_snapshot.as_deref(),
                occurred_at.as_str(),
                occurred_at.as_str(),
            ),
        )
        .map_err(|_| AiotEventRepositoryError::PersistenceFailure)?;

        tx.commit()
            .map_err(|_| AiotEventRepositoryError::PersistenceFailure)?;

        Ok(AiotDeviceEventRecord {
            id: (next_id + 1).to_string(),
            tenant_id: command.association.tenant_id,
            organization_id: command.association.organization_id,
            event_id,
            event_type: command.event_type,
            event_version: command.event_version,
            device_id: command.device_id,
            protocol_id: command.protocol_id,
            adapter_id: command.adapter_id,
            message_class: command.message_class,
            semantic_type: command.semantic_type,
            transport: command.transport,
            direction: command.direction,
            message_id: command.message_id,
            correlation_id: command.correlation_id,
            trace_id: command.trace_id,
            payload_hash: command.payload_hash,
            media_resource_id: command.media_resource_id,
            object_blob_id: command.object_blob_id,
            media_json: media_snapshot,
            payload_json: command.payload_json,
            occurred_at,
        })
    }

    fn list_events(
        &self,
        association: &AiotStorageAssociation,
        device_id: Option<&str>,
    ) -> Result<Vec<AiotDeviceEventRecord>, AiotEventRepositoryError> {
        let connection = self.connection.lock().expect("sqlite device repo poisoned");

        if let Some(scoped_device_id) = device_id {
            let mut statement = connection
                .prepare(
                    "SELECT id, uuid, device_id, event_type, event_payload, media_resource_id, object_blob_id, media_resource_snapshot, created_at FROM iot_device_event WHERE tenant_id = ?1 AND organization_id = ?2 AND device_id = ?3 ORDER BY id ASC",
                )
                .map_err(|_| AiotEventRepositoryError::PersistenceFailure)?;
            let rows = statement
                .query_map(
                    (
                        association.tenant_id,
                        association.organization_id,
                        scoped_device_id,
                    ),
                    |row| row_to_device_event_record(row, association),
                )
                .map_err(|_| AiotEventRepositoryError::PersistenceFailure)?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|_| AiotEventRepositoryError::PersistenceFailure)
        } else {
            let mut statement = connection
                .prepare(
                    "SELECT id, uuid, device_id, event_type, event_payload, media_resource_id, object_blob_id, media_resource_snapshot, created_at FROM iot_device_event WHERE tenant_id = ?1 AND organization_id = ?2 ORDER BY id ASC",
                )
                .map_err(|_| AiotEventRepositoryError::PersistenceFailure)?;
            let rows = statement
                .query_map(
                    (association.tenant_id, association.organization_id),
                    |row| row_to_device_event_record(row, association),
                )
                .map_err(|_| AiotEventRepositoryError::PersistenceFailure)?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|_| AiotEventRepositoryError::PersistenceFailure)
        }
    }
}

impl AiotDeviceTwinRepository for SqliteSqlxDeviceRepository {
    fn upsert_twin_property(
        &self,
        command: AiotTwinPropertyUpsertCommand,
    ) -> Result<AiotDeviceTwinSnapshot, AiotDeviceTwinRepositoryError> {
        let mut connection = self.connection.lock().expect("sqlite device repo poisoned");
        let tx = connection
            .transaction()
            .map_err(|_| AiotDeviceTwinRepositoryError::PersistenceFailure)?;

        ensure_twin_root_row(
            &tx,
            &command.association,
            &command.device_id,
            default_timestamp(),
        )
        .map_err(|_| AiotDeviceTwinRepositoryError::PersistenceFailure)?;

        let existing = tx
            .query_row(
                "SELECT id, desired_value, desired_version, reported_value, reported_version FROM iot_device_twin_property WHERE tenant_id = ?1 AND organization_id = ?2 AND device_id = ?3 AND property_name = ?4 LIMIT 1",
                (
                    command.association.tenant_id,
                    command.association.organization_id,
                    command.device_id.as_str(),
                    command.property_name.as_str(),
                ),
                |row| {
                    let id: i64 = row.get(0)?;
                    let desired_value: Option<String> = row.get(1)?;
                    let desired_version: i64 = row.get(2)?;
                    let reported_value: Option<String> = row.get(3)?;
                    let reported_version: i64 = row.get(4)?;
                    Ok((id, desired_value, desired_version, reported_value, reported_version))
                },
            )
            .ok();

        let desired_updated_at = command
            .desired_updated_at
            .as_deref()
            .unwrap_or(default_timestamp());
        let reported_updated_at = command
            .reported_updated_at
            .as_deref()
            .unwrap_or(default_timestamp());
        let updated_at = command
            .desired_updated_at
            .clone()
            .or(command.reported_updated_at.clone())
            .unwrap_or_else(|| default_timestamp().to_string());

        match existing {
            Some((
                id,
                existing_desired,
                existing_desired_version,
                existing_reported,
                existing_reported_version,
            )) => {
                let desired_value = command.desired_value_json.clone().or(existing_desired);
                let reported_value = command.reported_value_json.clone().or(existing_reported);
                let desired_version = if command.desired_value_json.is_some() {
                    existing_desired_version.saturating_add(1)
                } else {
                    existing_desired_version
                };
                let reported_version = if command.reported_value_json.is_some() {
                    existing_reported_version.saturating_add(1)
                } else {
                    existing_reported_version
                };
                tx.execute(
                    "UPDATE iot_device_twin_property SET desired_value = ?1, desired_version = ?2, desired_updated_at = ?3, reported_value = ?4, reported_version = ?5, reported_updated_at = ?6, updated_at = ?7 WHERE id = ?8",
                    (
                        desired_value.as_deref(),
                        desired_version,
                        desired_updated_at,
                        reported_value.as_deref(),
                        reported_version,
                        reported_updated_at,
                        updated_at.as_str(),
                        id,
                    ),
                )
                .map_err(|_| AiotDeviceTwinRepositoryError::PersistenceFailure)?;
            }
            None => {
                let next_property_id: i64 = tx
                    .query_row(
                        "SELECT COALESCE(MAX(id), 0) FROM iot_device_twin_property",
                        [],
                        |row| row.get(0),
                    )
                    .map_err(|_| AiotDeviceTwinRepositoryError::PersistenceFailure)?;
                tx.execute(
                    "INSERT INTO iot_device_twin_property (id, uuid, tenant_id, organization_id, data_scope, device_id, property_name, desired_value, desired_version, desired_updated_at, reported_value, reported_version, reported_updated_at, status, created_at, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 1, ?14, ?15, 0)",
                    rusqlite::params![
                        next_property_id + 1,
                        format!(
                            "twin-prop-{}-{}",
                            command.device_id,
                            command.property_name
                        ),
                        command.association.tenant_id,
                        command.association.organization_id,
                        command.association.data_scope as i64,
                        command.device_id.as_str(),
                        command.property_name.as_str(),
                        command.desired_value_json.as_deref(),
                        if command.desired_value_json.is_some() { 1 } else { 0 },
                        if command.desired_value_json.is_some() {
                            Some(desired_updated_at)
                        } else {
                            None
                        },
                        command.reported_value_json.as_deref(),
                        if command.reported_value_json.is_some() { 1 } else { 0 },
                        if command.reported_value_json.is_some() {
                            Some(reported_updated_at)
                        } else {
                            None
                        },
                        updated_at.as_str(),
                        updated_at.as_str(),
                    ],
                )
                .map_err(|_| AiotDeviceTwinRepositoryError::PersistenceFailure)?;
            }
        }

        recompute_twin_versions(&tx, &command.association, &command.device_id, &updated_at)
            .map_err(|_| AiotDeviceTwinRepositoryError::PersistenceFailure)?;

        tx.commit()
            .map_err(|_| AiotDeviceTwinRepositoryError::PersistenceFailure)?;

        drop(connection);
        self.get_twin_snapshot(&command.association, &command.device_id)
    }

    fn get_twin_snapshot(
        &self,
        association: &AiotStorageAssociation,
        device_id: &str,
    ) -> Result<AiotDeviceTwinSnapshot, AiotDeviceTwinRepositoryError> {
        let connection = self.connection.lock().expect("sqlite device repo poisoned");
        let mut desired = BTreeMap::new();
        let mut reported = BTreeMap::new();
        let mut statement = connection
            .prepare(
                "SELECT property_name, desired_value, reported_value FROM iot_device_twin_property WHERE tenant_id = ?1 AND organization_id = ?2 AND device_id = ?3 ORDER BY id ASC",
            )
            .map_err(|_| AiotDeviceTwinRepositoryError::PersistenceFailure)?;
        let rows = statement
            .query_map(
                (
                    association.tenant_id,
                    association.organization_id,
                    device_id,
                ),
                |row| {
                    let property_name: String = row.get(0)?;
                    let desired_value: Option<String> = row.get(1)?;
                    let reported_value: Option<String> = row.get(2)?;
                    Ok((property_name, desired_value, reported_value))
                },
            )
            .map_err(|_| AiotDeviceTwinRepositoryError::PersistenceFailure)?;
        for row in rows {
            let (property_name, desired_value, reported_value) =
                row.map_err(|_| AiotDeviceTwinRepositoryError::PersistenceFailure)?;
            if let Some(desired_value) = desired_value {
                desired.insert(property_name.clone(), desired_value);
            }
            if let Some(reported_value) = reported_value {
                reported.insert(property_name, reported_value);
            }
        }

        let twin_state = connection
            .query_row(
                "SELECT desired_version, reported_version, updated_at FROM iot_device_twin WHERE tenant_id = ?1 AND organization_id = ?2 AND device_id = ?3 LIMIT 1",
                (association.tenant_id, association.organization_id, device_id),
                |row| {
                    let desired_version: i64 = row.get(0)?;
                    let reported_version: i64 = row.get(1)?;
                    let updated_at: Option<String> = row.get(2)?;
                    Ok((desired_version, reported_version, updated_at))
                },
            )
            .ok();

        let (desired_version, reported_version, updated_at) =
            twin_state.unwrap_or((0, 0, Some(default_timestamp().to_string())));

        Ok(AiotDeviceTwinSnapshot {
            tenant_id: association.tenant_id,
            organization_id: association.organization_id,
            device_id: device_id.to_string(),
            desired,
            reported,
            desired_version,
            reported_version,
            updated_at: updated_at.unwrap_or_else(|| default_timestamp().to_string()),
        })
    }
}

fn ensure_device_schema(connection: &Connection) -> Result<(), rusqlite::Error> {
    let table_exists: i64 = connection.query_row(
        "SELECT COUNT(1) FROM sqlite_master WHERE type = 'table' AND name = 'iot_device'",
        [],
        |row| row.get(0),
    )?;
    if table_exists == 0 {
        connection.execute_batch(initial_migration_sql())?;
    }
    Ok(())
}

fn execute_sql_statement(
    tx: &rusqlite::Transaction<'_>,
    statement: &SqlStatementPlan,
) -> Result<usize, rusqlite::Error> {
    let values = statement
        .binds
        .iter()
        .map(sql_bind_value_to_sqlite_value)
        .collect::<Vec<_>>();
    tx.execute(&statement.sql, params_from_iter(values.iter()))
}

fn sql_bind_value_to_sqlite_value(value: &SqlBindValue) -> SqliteValue {
    match value {
        SqlBindValue::Text(value) => SqliteValue::Text(value.clone()),
        SqlBindValue::Int64(value) => SqliteValue::Integer(*value),
        SqlBindValue::Null => SqliteValue::Null,
    }
}

fn device_status_text(status: i64) -> String {
    match status {
        0 => "inactive".to_string(),
        1 => "active".to_string(),
        2 => "disabled".to_string(),
        3 => "deleted".to_string(),
        _ => "active".to_string(),
    }
}

fn command_status_code(status: &str) -> i64 {
    match status {
        "accepted" => 1,
        "dispatched" => 2,
        "acknowledged" => 3,
        "succeeded" => 4,
        "failed" => 5,
        "cancelled" => 6,
        "timeout" => 7,
        _ => 0,
    }
}

fn command_status_text(status: i64) -> String {
    match status {
        1 => "accepted".to_string(),
        2 => "dispatched".to_string(),
        3 => "acknowledged".to_string(),
        4 => "succeeded".to_string(),
        5 => "failed".to_string(),
        6 => "cancelled".to_string(),
        7 => "timeout".to_string(),
        _ => "pending".to_string(),
    }
}

fn default_timestamp() -> &'static str {
    "2026-06-01T00:00:00Z"
}

fn scoped_command_key(association: &AiotStorageAssociation, command_id: &str) -> String {
    format!(
        "{}:{}:{}",
        association.tenant_id, association.organization_id, command_id
    )
}

fn scoped_device_session_key(
    association: &AiotStorageAssociation,
    device_id: &str,
    session_id: &str,
) -> String {
    format!(
        "{}:{}:{}:{}",
        association.tenant_id, association.organization_id, device_id, session_id
    )
}

fn empty_twin_snapshot(
    association: &AiotStorageAssociation,
    device_id: &str,
) -> AiotDeviceTwinSnapshot {
    AiotDeviceTwinSnapshot {
        tenant_id: association.tenant_id,
        organization_id: association.organization_id,
        device_id: device_id.to_string(),
        desired: BTreeMap::new(),
        reported: BTreeMap::new(),
        desired_version: 0,
        reported_version: 0,
        updated_at: default_timestamp().to_string(),
    }
}

fn command_result_for(
    connection: &Connection,
    tenant_id: i64,
    organization_id: i64,
    command_id: &str,
) -> Result<Option<AiotCommandResultRecord>, AiotCommandRepositoryError> {
    connection
        .query_row(
            "SELECT result_code, result_payload, result_media_resource_id, result_object_blob_id, result_media_resource_snapshot, updated_at FROM iot_command_result WHERE tenant_id = ?1 AND organization_id = ?2 AND command_id = ?3 ORDER BY id DESC LIMIT 1",
            (tenant_id, organization_id, command_id),
            |row| {
                let result_code: Option<String> = row.get(0)?;
                let result_payload_json: Option<String> = row.get(1)?;
                let result_media_resource_id: Option<String> = row.get(2)?;
                let result_object_blob_id: Option<String> = row.get(3)?;
                let result_media_json: Option<String> = row.get(4)?;
                let occurred_at: Option<String> = row.get(5)?;
                Ok(AiotCommandResultRecord {
                    result_code,
                    result_payload_json,
                    result_media_resource_id,
                    result_object_blob_id,
                    result_media_json,
                    occurred_at,
                })
            },
        )
        .map(Some)
        .or_else(|error| {
            if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                Ok(None)
            } else {
                Err(AiotCommandRepositoryError::PersistenceFailure)
            }
        })
}

fn row_to_device_event_record(
    row: &rusqlite::Row<'_>,
    association: &AiotStorageAssociation,
) -> rusqlite::Result<AiotDeviceEventRecord> {
    let id: i64 = row.get(0)?;
    let event_id: String = row.get(1)?;
    let device_id: String = row.get(2)?;
    let event_type: String = row.get(3)?;
    let event_payload_json: String = row.get(4)?;
    let media_resource_id: Option<String> = row.get(5)?;
    let object_blob_id: Option<String> = row.get(6)?;
    let media_json: Option<String> = row.get(7)?;
    let created_at: Option<String> = row.get(8)?;
    let parsed_payload = serde_json::from_str::<JsonValue>(&event_payload_json).ok();

    let envelope = parsed_payload.as_ref().and_then(JsonValue::as_object);
    let payload_json = envelope
        .and_then(|payload| payload.get("payload"))
        .map(JsonValue::to_string)
        .unwrap_or(event_payload_json);

    let event_version = envelope
        .and_then(|payload| payload.get("eventVersion"))
        .and_then(JsonValue::as_str)
        .unwrap_or("1")
        .to_string();
    let protocol_id = envelope
        .and_then(|payload| payload.get("protocolId"))
        .and_then(JsonValue::as_str)
        .unwrap_or("xiaozhi.websocket")
        .to_string();
    let adapter_id = envelope
        .and_then(|payload| payload.get("adapterId"))
        .and_then(JsonValue::as_str)
        .unwrap_or("xiaozhi")
        .to_string();
    let message_class = envelope
        .and_then(|payload| payload.get("messageClass"))
        .and_then(JsonValue::as_str)
        .unwrap_or("mediaFrame")
        .to_string();
    let semantic_type = envelope
        .and_then(|payload| payload.get("semanticType"))
        .and_then(JsonValue::as_str)
        .unwrap_or("audio")
        .to_string();
    let transport = envelope
        .and_then(|payload| payload.get("transport"))
        .and_then(JsonValue::as_str)
        .unwrap_or("websocket")
        .to_string();
    let direction = envelope
        .and_then(|payload| payload.get("direction"))
        .and_then(JsonValue::as_str)
        .unwrap_or("device_to_cloud")
        .to_string();
    let message_id = envelope
        .and_then(|payload| payload.get("messageId"))
        .and_then(JsonValue::as_str)
        .map(str::to_string);
    let correlation_id = envelope
        .and_then(|payload| payload.get("correlationId"))
        .and_then(JsonValue::as_str)
        .map(str::to_string);
    let trace_id = envelope
        .and_then(|payload| payload.get("traceId"))
        .and_then(JsonValue::as_str)
        .map(str::to_string);
    let payload_hash = envelope
        .and_then(|payload| payload.get("payloadHash"))
        .and_then(JsonValue::as_str)
        .map(str::to_string);
    let occurred_at = envelope
        .and_then(|payload| payload.get("occurredAt"))
        .and_then(JsonValue::as_str)
        .map(str::to_string)
        .or(created_at)
        .unwrap_or_else(|| default_timestamp().to_string());

    Ok(AiotDeviceEventRecord {
        id: id.to_string(),
        tenant_id: association.tenant_id,
        organization_id: association.organization_id,
        event_id,
        event_type,
        event_version,
        device_id,
        protocol_id,
        adapter_id,
        message_class,
        semantic_type,
        transport,
        direction,
        message_id,
        correlation_id,
        trace_id,
        payload_hash,
        media_resource_id,
        object_blob_id,
        media_json,
        payload_json,
        occurred_at,
    })
}

fn ensure_twin_root_row(
    tx: &rusqlite::Transaction<'_>,
    association: &AiotStorageAssociation,
    device_id: &str,
    updated_at: &str,
) -> Result<(), rusqlite::Error> {
    let existing: i64 = tx.query_row(
        "SELECT COUNT(1) FROM iot_device_twin WHERE tenant_id = ?1 AND organization_id = ?2 AND device_id = ?3",
        (association.tenant_id, association.organization_id, device_id),
        |row| row.get(0),
    )?;
    if existing > 0 {
        return Ok(());
    }

    let next_twin_id: i64 = tx.query_row(
        "SELECT COALESCE(MAX(id), 0) FROM iot_device_twin",
        [],
        |row| row.get(0),
    )?;
    tx.execute(
        "INSERT INTO iot_device_twin (id, uuid, tenant_id, organization_id, data_scope, device_id, desired_version, reported_version, status, created_at, updated_at, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, 1, ?7, ?8, 0)",
        (
            next_twin_id + 1,
            format!("twin-{}", device_id),
            association.tenant_id,
            association.organization_id,
            association.data_scope as i64,
            device_id,
            updated_at,
            updated_at,
        ),
    )?;
    Ok(())
}

fn recompute_twin_versions(
    tx: &rusqlite::Transaction<'_>,
    association: &AiotStorageAssociation,
    device_id: &str,
    updated_at: &str,
) -> Result<(), rusqlite::Error> {
    let (desired_version, reported_version): (i64, i64) = tx.query_row(
        "SELECT COALESCE(MAX(desired_version), 0), COALESCE(MAX(reported_version), 0) FROM iot_device_twin_property WHERE tenant_id = ?1 AND organization_id = ?2 AND device_id = ?3",
        (association.tenant_id, association.organization_id, device_id),
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    tx.execute(
        "UPDATE iot_device_twin SET desired_version = ?1, reported_version = ?2, updated_at = ?3 WHERE tenant_id = ?4 AND organization_id = ?5 AND device_id = ?6",
        (
            desired_version,
            reported_version,
            updated_at,
            association.tenant_id,
            association.organization_id,
            device_id,
        ),
    )?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlBindValue {
    Text(String),
    Int64(i64),
    Null,
}

impl SqlBindValue {
    fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    fn optional_text(value: Option<&str>) -> Self {
        value
            .map(|value| Self::Text(value.to_string()))
            .unwrap_or(Self::Null)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDialect {
    Postgres,
    Sqlite,
}

impl SqlDialect {
    fn placeholder(&self, index: usize) -> String {
        match self {
            Self::Postgres => format!("${index}"),
            Self::Sqlite => "?".to_string(),
        }
    }

    fn placeholders(&self, count: usize) -> String {
        match self {
            Self::Postgres => (1..=count)
                .map(|index| self.placeholder(index))
                .collect::<Vec<_>>()
                .join(", "),
            Self::Sqlite => vec!["?"; count].join(", "),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlPlanError {
    pub code: String,
    pub table: Option<String>,
    pub column: Option<String>,
    pub statement_kind: Option<&'static str>,
}

impl SqlPlanError {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            table: None,
            column: None,
            statement_kind: None,
        }
    }

    pub fn with_table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    pub fn with_statement_kind(mut self, statement_kind: &'static str) -> Self {
        self.statement_kind = Some(statement_kind);
        self
    }

    pub fn with_column(mut self, column: impl Into<String>) -> Self {
        self.column = Some(column.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlStatementPlan {
    pub statement_kind: &'static str,
    pub table: &'static str,
    pub dialect: SqlDialect,
    pub sql: String,
    pub binds: Vec<SqlBindValue>,
}

impl SqlStatementPlan {
    pub fn new(statement_kind: &'static str, table: &'static str, sql: impl Into<String>) -> Self {
        Self {
            statement_kind,
            table,
            dialect: SqlDialect::Postgres,
            sql: sql.into(),
            binds: Vec::new(),
        }
    }

    pub fn with_dialect(mut self, dialect: SqlDialect) -> Self {
        self.dialect = dialect;
        self
    }

    pub fn with_binds(mut self, binds: Vec<SqlBindValue>) -> Self {
        self.binds = binds;
        self
    }

    pub fn placeholder_count(&self) -> usize {
        match self.dialect {
            SqlDialect::Postgres => postgres_placeholder_count(&self.sql),
            SqlDialect::Sqlite => self
                .sql
                .chars()
                .filter(|candidate| *candidate == '?')
                .count(),
        }
    }

    pub fn validate(&self) -> Result<(), SqlPlanError> {
        let placeholder_count = self.placeholder_count();
        if placeholder_count != self.binds.len() {
            return Err(SqlPlanError::new("storage.sql.bind_count_mismatch")
                .with_table(self.table)
                .with_statement_kind(self.statement_kind));
        }

        if table_contract(self.table).is_none() {
            return Err(SqlPlanError::new("storage.sql.table.unsupported")
                .with_table(self.table)
                .with_statement_kind(self.statement_kind));
        }

        for column in sql_write_columns(&self.sql) {
            if !initial_migration_declares_column(self.table, &column) {
                return Err(SqlPlanError::new("storage.sql.column.unsupported")
                    .with_table(self.table)
                    .with_column(column)
                    .with_statement_kind(self.statement_kind));
            }
        }

        Ok(())
    }
}

impl SqlStatementPlan {
    fn bound(
        statement_kind: &'static str,
        table: &'static str,
        dialect: SqlDialect,
        sql: impl Into<String>,
        binds: Vec<SqlBindValue>,
    ) -> Self {
        Self::new(statement_kind, table, sql)
            .with_dialect(dialect)
            .with_binds(binds)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlStatementBatch {
    pub batch_kind: &'static str,
    pub statements: Vec<SqlStatementPlan>,
}

impl SqlStatementBatch {
    pub fn new(batch_kind: &'static str, statements: Vec<SqlStatementPlan>) -> Self {
        Self {
            batch_kind,
            statements,
        }
    }

    pub fn single(batch_kind: &'static str, statement: SqlStatementPlan) -> Self {
        Self::new(batch_kind, vec![statement])
    }

    pub fn validate(&self) -> Result<(), SqlPlanError> {
        for statement in &self.statements {
            statement.validate()?;
        }

        Ok(())
    }
}

pub trait SqlStatementExecutor: Clone {
    fn execute_idempotency_guard(&self, key: &str, statement: SqlStatementPlan) -> bool;

    fn execute_batch(&self, batch: SqlStatementBatch);

    fn execute_transaction(&self, transaction: SqlTransactionPlan) -> SqlTransactionOutcome {
        let SqlTransactionPlan {
            idempotency_key,
            guard,
            write_batch,
            ..
        } = transaction;

        if !self.execute_idempotency_guard(&idempotency_key, guard) {
            return SqlTransactionOutcome::Duplicate;
        }

        self.execute_batch(write_batch);
        SqlTransactionOutcome::Committed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlProtocolCommandPlan {
    pub idempotency_key: String,
    pub guard: SqlStatementPlan,
    pub write_batch: SqlStatementBatch,
}

impl SqlProtocolCommandPlan {
    pub fn validate(&self) -> Result<(), SqlPlanError> {
        self.guard.validate()?;
        self.write_batch.validate()?;

        Ok(())
    }

    pub fn into_transaction_plan(self) -> SqlTransactionPlan {
        SqlTransactionPlan::new(
            "protocol_ingest",
            self.idempotency_key,
            self.guard,
            self.write_batch,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlTransactionFailurePolicy {
    RollbackAll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlTransactionOutcome {
    Committed,
    Duplicate,
    RolledBack { reason_code: String },
}

impl SqlTransactionOutcome {
    pub fn rolled_back(reason_code: impl Into<String>) -> Self {
        Self::RolledBack {
            reason_code: reason_code.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlTransactionPlan {
    pub transaction_kind: &'static str,
    pub failure_policy: SqlTransactionFailurePolicy,
    pub idempotency_key: String,
    pub guard: SqlStatementPlan,
    pub write_batch: SqlStatementBatch,
}

impl SqlTransactionPlan {
    pub fn new(
        transaction_kind: &'static str,
        idempotency_key: impl Into<String>,
        guard: SqlStatementPlan,
        write_batch: SqlStatementBatch,
    ) -> Self {
        Self {
            transaction_kind,
            failure_policy: SqlTransactionFailurePolicy::RollbackAll,
            idempotency_key: idempotency_key.into(),
            guard,
            write_batch,
        }
    }

    pub fn ordered_statements(&self) -> Vec<SqlStatementPlan> {
        let mut statements = Vec::with_capacity(1 + self.write_batch.statements.len());
        statements.push(self.guard.clone());
        statements.extend(self.write_batch.statements.iter().cloned());
        statements
    }

    pub fn validate(&self) -> Result<(), SqlPlanError> {
        self.guard.validate()?;
        self.write_batch.validate()?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SqlProtocolIngestPlanner {
    dialect: SqlDialect,
}

impl SqlProtocolIngestPlanner {
    pub fn standard() -> Self {
        Self::for_dialect(SqlDialect::Postgres)
    }

    pub fn for_dialect(dialect: SqlDialect) -> Self {
        Self { dialect }
    }

    pub fn dialect(&self) -> SqlDialect {
        self.dialect
    }

    pub fn plan_protocol_command(
        &self,
        command: &AiotProtocolStorageCommand,
    ) -> SqlProtocolCommandPlan {
        self.try_plan_protocol_command(command)
            .expect("standard protocol command plan must be valid")
    }

    pub fn try_plan_protocol_command(
        &self,
        command: &AiotProtocolStorageCommand,
    ) -> Result<SqlProtocolCommandPlan, SqlPlanError> {
        if table_contract(command.primary_table).is_none() {
            return Err(SqlPlanError::new("storage.sql.primary_table.unsupported")
                .with_table(command.primary_table));
        }

        let idempotency_key = command.idempotency_key.clone().unwrap_or_else(|| {
            format!(
                "{}:{}:{}:{}:{}",
                command.protocol_id,
                command.adapter_id,
                command.device_id,
                command.kind.as_str(),
                command.primary_table
            )
        });
        let guard = idempotency_guard_statement(self.dialect, command, &idempotency_key);
        let mut statements = vec![primary_write_statement(
            self.dialect,
            command,
            &idempotency_key,
        )];
        if command.outbox.is_some() {
            statements.push(outbox_write_statement(self.dialect, command));
        }

        let plan = SqlProtocolCommandPlan {
            idempotency_key,
            guard,
            write_batch: SqlStatementBatch::new("protocol_ingest_write", statements),
        };
        plan.validate()?;

        Ok(plan)
    }

    pub fn plan_dead_letter(&self, intent: &AiotProtocolDeadLetterIntent) -> SqlStatementBatch {
        self.try_plan_dead_letter(intent)
            .expect("standard dead-letter plan must be valid")
    }

    pub fn try_plan_dead_letter(
        &self,
        intent: &AiotProtocolDeadLetterIntent,
    ) -> Result<SqlStatementBatch, SqlPlanError> {
        let batch = SqlStatementBatch::single(
            "dead_letter_write",
            dead_letter_write_statement(self.dialect, intent),
        );
        batch.validate()?;

        Ok(batch)
    }
}

impl Default for SqlProtocolIngestPlanner {
    fn default() -> Self {
        Self::standard()
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemorySqlStatementExecutor {
    state: Arc<Mutex<InMemorySqlStatementExecutorState>>,
}

#[derive(Debug, Default)]
struct InMemorySqlStatementExecutorState {
    idempotency_keys: BTreeSet<String>,
    executed_statements: Vec<SqlStatementPlan>,
    executed_batches: Vec<SqlStatementBatch>,
}

impl InMemorySqlStatementExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn claim_idempotency_key(&self, key: &str) -> bool {
        self.state
            .lock()
            .expect("sql statement executor poisoned")
            .idempotency_keys
            .insert(key.to_string())
    }

    pub fn execute(&self, statement: SqlStatementPlan) {
        self.state
            .lock()
            .expect("sql statement executor poisoned")
            .executed_statements
            .push(statement);
    }

    pub fn execute_batch(&self, batch: SqlStatementBatch) {
        let mut state = self.state.lock().expect("sql statement executor poisoned");
        state
            .executed_statements
            .extend(batch.statements.iter().cloned());
        state.executed_batches.push(batch);
    }

    pub fn executed_statements(&self) -> Vec<SqlStatementPlan> {
        self.state
            .lock()
            .expect("sql statement executor poisoned")
            .executed_statements
            .clone()
    }

    pub fn executed_batches(&self) -> Vec<SqlStatementBatch> {
        self.state
            .lock()
            .expect("sql statement executor poisoned")
            .executed_batches
            .clone()
    }
}

impl SqlStatementExecutor for InMemorySqlStatementExecutor {
    fn execute_idempotency_guard(&self, key: &str, statement: SqlStatementPlan) -> bool {
        let mut state = self.state.lock().expect("sql statement executor poisoned");
        state.executed_statements.push(statement.clone());
        state
            .executed_batches
            .push(SqlStatementBatch::single("idempotency_guard", statement));
        state.idempotency_keys.insert(key.to_string())
    }

    fn execute_batch(&self, batch: SqlStatementBatch) {
        InMemorySqlStatementExecutor::execute_batch(self, batch);
    }
}

#[derive(Debug, Clone)]
pub struct SqlxProtocolIngestUnitOfWork<E: SqlStatementExecutor = InMemorySqlStatementExecutor> {
    executor: E,
    planner: SqlProtocolIngestPlanner,
}

impl<E: SqlStatementExecutor> SqlxProtocolIngestUnitOfWork<E> {
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            planner: SqlProtocolIngestPlanner::standard(),
        }
    }

    pub fn with_planner(executor: E, planner: SqlProtocolIngestPlanner) -> Self {
        Self { executor, planner }
    }
}

impl<E: SqlStatementExecutor> AiotProtocolIngestUnitOfWork for SqlxProtocolIngestUnitOfWork<E> {
    fn execute_protocol_command(
        &self,
        command: &AiotProtocolStorageCommand,
    ) -> AiotStorageWriteReceipt {
        let plan = match self.planner.try_plan_protocol_command(command) {
            Ok(plan) => plan,
            Err(error) => return AiotStorageWriteReceipt::dead_lettered(error.code),
        };
        let outcome = self
            .executor
            .execute_transaction(plan.into_transaction_plan());

        match outcome {
            SqlTransactionOutcome::Committed => AiotStorageWriteReceipt::accepted(
                command.kind,
                command.primary_table,
                command
                    .outbox
                    .as_ref()
                    .map(|outbox| outbox.event_type.clone()),
            ),
            SqlTransactionOutcome::Duplicate => {
                let mut receipt = AiotStorageWriteReceipt::accepted(
                    command.kind,
                    command.primary_table,
                    command
                        .outbox
                        .as_ref()
                        .map(|outbox| outbox.event_type.clone()),
                );
                receipt.duplicate = true;
                receipt
            }
            SqlTransactionOutcome::RolledBack { reason_code } => {
                AiotStorageWriteReceipt::dead_lettered(reason_code)
            }
        }
    }

    fn record_dead_letter(&self, intent: &AiotProtocolDeadLetterIntent) -> AiotStorageWriteReceipt {
        let batch = match self.planner.try_plan_dead_letter(intent) {
            Ok(batch) => batch,
            Err(error) => return AiotStorageWriteReceipt::dead_lettered(error.code),
        };
        self.executor.execute_batch(batch);
        AiotStorageWriteReceipt::dead_lettered(intent.reason_code.clone())
    }
}

fn postgres_placeholder_count(sql: &str) -> usize {
    let mut max_placeholder = 0;
    let bytes = sql.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'$' {
            let mut number_index = index + 1;
            let mut value = 0_usize;
            while number_index < bytes.len() && bytes[number_index].is_ascii_digit() {
                value = value
                    .saturating_mul(10)
                    .saturating_add((bytes[number_index] - b'0') as usize);
                number_index += 1;
            }
            if number_index > index + 1 {
                max_placeholder = max_placeholder.max(value);
                index = number_index;
                continue;
            }
        }
        index += 1;
    }

    max_placeholder
}

fn sql_write_columns(sql: &str) -> Vec<String> {
    let trimmed = sql.trim_start();
    let upper = trimmed.to_ascii_uppercase();

    if upper.starts_with("INSERT INTO ") {
        return insert_write_columns(trimmed);
    }

    if upper.starts_with("UPDATE ") {
        return update_write_columns(trimmed);
    }

    Vec::new()
}

fn insert_write_columns(sql: &str) -> Vec<String> {
    let Some(start) = sql.find('(') else {
        return Vec::new();
    };
    let Some(end) = sql[start + 1..].find(')') else {
        return Vec::new();
    };

    comma_separated_identifiers(&sql[start + 1..start + 1 + end])
}

fn update_write_columns(sql: &str) -> Vec<String> {
    let Some(set_start) = find_ascii_case_insensitive(sql, " SET ") else {
        return Vec::new();
    };
    let after_set = set_start + " SET ".len();
    let where_start = find_ascii_case_insensitive(&sql[after_set..], " WHERE ")
        .map(|offset| after_set + offset)
        .unwrap_or(sql.len());

    sql[after_set..where_start]
        .split(',')
        .filter_map(|assignment| assignment.split_once('='))
        .map(|(column, _)| normalize_sql_identifier(column))
        .filter(|column| !column.is_empty())
        .collect()
}

fn comma_separated_identifiers(segment: &str) -> Vec<String> {
    segment
        .split(',')
        .map(normalize_sql_identifier)
        .filter(|column| !column.is_empty())
        .collect()
}

fn normalize_sql_identifier(identifier: &str) -> String {
    identifier
        .trim()
        .trim_matches('"')
        .trim_matches('`')
        .trim_matches('[')
        .trim_matches(']')
        .to_string()
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .to_ascii_uppercase()
        .find(&needle.to_ascii_uppercase())
}

fn initial_migration_declares_column(table: &str, column: &str) -> bool {
    let Some(definition) = initial_migration_table_definition(table) else {
        return false;
    };

    definition
        .lines()
        .map(str::trim)
        .any(|line| line.starts_with(&format!("{column} ")))
}

fn initial_migration_table_definition(table: &str) -> Option<&'static str> {
    let sql = initial_migration_sql();
    let marker = format!("CREATE TABLE {table}");
    let start = sql.find(&marker)?;
    let rest = &sql[start + marker.len()..];
    let end = rest.find("\nCREATE TABLE ").unwrap_or(rest.len());

    Some(&sql[start..start + marker.len() + end])
}

fn device_create_statement(
    dialect: SqlDialect,
    device: &AiotDeviceRecord,
) -> Result<SqlStatementPlan, SqlPlanError> {
    let id = device
        .id
        .parse::<i64>()
        .map_err(|_| SqlPlanError::new("storage.sql.device.invalid_id").with_table("iot_device"))?;
    let product_id = parse_device_product_id(&device.product_id)?;
    let status = device_status_code(&device.status);
    let owner_id = format!(
        "{}:{}:{}",
        device.tenant_id, device.organization_id, device.device_id
    );
    let statement = SqlStatementPlan::bound(
        "device_create",
        "iot_device",
        dialect,
        format!(
            "INSERT INTO iot_device (id, uuid, tenant_id, organization_id, data_scope, owner_type, owner_id, device_key, product_id, display_name, device_id, client_id, chip_family, lifecycle_state, last_seen_at, metadata, status, created_at, updated_at, version, created_by, updated_by) VALUES ({})",
            dialect.placeholders(22)
        ),
        vec![
            SqlBindValue::Int64(id),
            SqlBindValue::text(format!("iot-device-{id}")),
            SqlBindValue::Int64(device.tenant_id),
            SqlBindValue::Int64(device.organization_id),
            SqlBindValue::Int64(0),
            SqlBindValue::text("device"),
            SqlBindValue::text(owner_id),
            SqlBindValue::text(&device.device_id),
            SqlBindValue::Int64(product_id),
            SqlBindValue::text(&device.display_name),
            SqlBindValue::text(&device.device_id),
            SqlBindValue::optional_text(device.client_id.as_deref()),
            SqlBindValue::optional_text(device.chip_family.as_deref()),
            SqlBindValue::Int64(0),
            SqlBindValue::optional_text(Some(&device.last_seen_at)),
            SqlBindValue::optional_text(device.metadata_json.as_deref()),
            SqlBindValue::Int64(status),
            SqlBindValue::text("2026-01-01T00:00:00Z"),
            SqlBindValue::text("2026-01-01T00:00:00Z"),
            SqlBindValue::Int64(0),
            SqlBindValue::Null,
            SqlBindValue::Null,
        ],
    );
    Ok(statement)
}

fn device_update_statement(
    dialect: SqlDialect,
    device: &AiotDeviceRecord,
) -> Result<SqlStatementPlan, SqlPlanError> {
    let status = device_status_code(&device.status);
    let statement = SqlStatementPlan::bound(
        "device_update",
        "iot_device",
        dialect,
        format!(
            "UPDATE iot_device SET display_name = {}, client_id = {}, chip_family = {}, status = {}, metadata = {}, updated_at = {} WHERE tenant_id = {} AND organization_id = {} AND device_id = {}",
            dialect.placeholder(1),
            dialect.placeholder(2),
            dialect.placeholder(3),
            dialect.placeholder(4),
            dialect.placeholder(5),
            dialect.placeholder(6),
            dialect.placeholder(7),
            dialect.placeholder(8),
            dialect.placeholder(9)
        ),
        vec![
            SqlBindValue::text(&device.display_name),
            SqlBindValue::optional_text(device.client_id.as_deref()),
            SqlBindValue::optional_text(device.chip_family.as_deref()),
            SqlBindValue::Int64(status),
            SqlBindValue::optional_text(device.metadata_json.as_deref()),
            SqlBindValue::text("2026-01-01T00:00:00Z"),
            SqlBindValue::Int64(device.tenant_id),
            SqlBindValue::Int64(device.organization_id),
            SqlBindValue::text(&device.device_id),
        ],
    );
    Ok(statement)
}

fn device_delete_statement(
    dialect: SqlDialect,
    association: &AiotStorageAssociation,
    device_id: &str,
) -> SqlStatementPlan {
    SqlStatementPlan::bound(
        "device_delete",
        "iot_device",
        dialect,
        format!(
            "DELETE FROM iot_device WHERE tenant_id = {} AND organization_id = {} AND device_id = {}",
            dialect.placeholder(1),
            dialect.placeholder(2),
            dialect.placeholder(3)
        ),
        vec![
            SqlBindValue::Int64(association.tenant_id),
            SqlBindValue::Int64(association.organization_id),
            SqlBindValue::text(device_id),
        ],
    )
}

fn device_status_code(status: &str) -> i64 {
    match status {
        "inactive" => 0,
        "active" => 1,
        "disabled" => 2,
        "deleted" => 3,
        _ => 1,
    }
}

fn parse_device_product_id(value: &str) -> Result<i64, SqlPlanError> {
    if value.is_empty() || !value.as_bytes().iter().all(u8::is_ascii_digit) {
        return Err(
            SqlPlanError::new("storage.sql.device.invalid_product_id").with_table("iot_device")
        );
    }

    value.parse::<i64>().map_err(|_| {
        SqlPlanError::new("storage.sql.device.invalid_product_id").with_table("iot_device")
    })
}

fn idempotency_guard_statement(
    dialect: SqlDialect,
    command: &AiotProtocolStorageCommand,
    idempotency_key: &str,
) -> SqlStatementPlan {
    SqlStatementPlan::bound(
        "idempotency_guard",
        "iot_protocol_ingest_record",
        dialect,
        format!(
            "INSERT INTO iot_protocol_ingest_record (tenant_id, organization_id, data_scope, protocol_id, adapter_id, device_id, message_id, correlation_id, media_resource_id, object_blob_id, media_resource_snapshot, idempotency_key, trace_id, status) VALUES ({}) ON CONFLICT DO NOTHING",
            dialect.placeholders(14)
        ),
        vec![
            SqlBindValue::Int64(command.association.tenant_id),
            SqlBindValue::Int64(command.association.organization_id),
            SqlBindValue::Int64(command.association.data_scope.into()),
            SqlBindValue::text(&command.protocol_id),
            SqlBindValue::text(&command.adapter_id),
            SqlBindValue::text(&command.device_id),
            SqlBindValue::optional_text(command.message_id.as_deref()),
            SqlBindValue::optional_text(command.correlation_id.as_deref()),
            SqlBindValue::optional_text(command.media_resource_id.as_deref()),
            SqlBindValue::optional_text(command.object_blob_id.as_deref()),
            SqlBindValue::optional_text(command.media_resource_snapshot.as_deref()),
            SqlBindValue::text(idempotency_key),
            SqlBindValue::optional_text(command.trace_id.as_deref()),
            SqlBindValue::Int64(0),
        ],
    )
}

fn primary_write_statement(
    dialect: SqlDialect,
    command: &AiotProtocolStorageCommand,
    idempotency_key: &str,
) -> SqlStatementPlan {
    let placeholders = (1..=11)
        .map(|index| dialect.placeholder(index))
        .collect::<Vec<_>>();

    SqlStatementPlan::bound(
        "primary_write",
        "iot_protocol_ingest_record",
        dialect,
        format!(
            "UPDATE iot_protocol_ingest_record SET status = {}, media_resource_id = {}, object_blob_id = {}, media_resource_snapshot = {} WHERE tenant_id = {} AND organization_id = {} AND data_scope = {} AND protocol_id = {} AND adapter_id = {} AND device_id = {} AND idempotency_key = {}",
            placeholders[0],
            placeholders[1],
            placeholders[2],
            placeholders[3],
            placeholders[4],
            placeholders[5],
            placeholders[6],
            placeholders[7],
            placeholders[8],
            placeholders[9],
            placeholders[10]
        ),
        vec![
            SqlBindValue::Int64(1),
            SqlBindValue::optional_text(command.media_resource_id.as_deref()),
            SqlBindValue::optional_text(command.object_blob_id.as_deref()),
            SqlBindValue::optional_text(command.media_resource_snapshot.as_deref()),
            SqlBindValue::Int64(command.association.tenant_id),
            SqlBindValue::Int64(command.association.organization_id),
            SqlBindValue::Int64(command.association.data_scope.into()),
            SqlBindValue::text(&command.protocol_id),
            SqlBindValue::text(&command.adapter_id),
            SqlBindValue::text(&command.device_id),
            SqlBindValue::text(idempotency_key),
        ],
    )
}

fn outbox_write_statement(
    dialect: SqlDialect,
    command: &AiotProtocolStorageCommand,
) -> SqlStatementPlan {
    let outbox = command.outbox.as_ref().expect("outbox intent");
    SqlStatementPlan::bound(
        "outbox_write",
        "iot_outbox_event",
        dialect,
        format!(
            "INSERT INTO iot_outbox_event (tenant_id, organization_id, data_scope, event_id, event_type, event_version, aggregate_type, aggregate_id, payload, payload_hash, status, trace_id, attempt_count) VALUES ({})",
            dialect.placeholders(13)
        ),
        vec![
            SqlBindValue::Int64(command.association.tenant_id),
            SqlBindValue::Int64(command.association.organization_id),
            SqlBindValue::Int64(command.association.data_scope.into()),
            SqlBindValue::text(format!(
                "{}:{}:{}",
                outbox.aggregate_type, outbox.aggregate_id, outbox.event_type
            )),
            SqlBindValue::text(&outbox.event_type),
            SqlBindValue::text(&outbox.event_version),
            SqlBindValue::text(&outbox.aggregate_type),
            SqlBindValue::text(&outbox.aggregate_id),
            SqlBindValue::text(&outbox.payload_json),
            SqlBindValue::optional_text(outbox.payload_hash.as_deref()),
            SqlBindValue::Int64(0),
            SqlBindValue::optional_text(command.trace_id.as_deref()),
            SqlBindValue::Int64(0),
        ],
    )
}

fn dead_letter_write_statement(
    dialect: SqlDialect,
    intent: &AiotProtocolDeadLetterIntent,
) -> SqlStatementPlan {
    SqlStatementPlan::bound(
        "dead_letter_write",
        "iot_protocol_message_dead_letter",
        dialect,
        format!(
            "INSERT INTO iot_protocol_message_dead_letter (tenant_id, organization_id, data_scope, protocol_id, adapter_id, device_id, reason_code, payload_ref, payload_hash, trace_id, status) VALUES ({})",
            dialect.placeholders(11)
        ),
        vec![
            SqlBindValue::Int64(intent.association.tenant_id),
            SqlBindValue::Int64(intent.association.organization_id),
            SqlBindValue::Int64(intent.association.data_scope.into()),
            SqlBindValue::text(&intent.protocol_id),
            SqlBindValue::text(&intent.adapter_id),
            SqlBindValue::optional_text(intent.device_id.as_deref()),
            SqlBindValue::text(&intent.reason_code),
            SqlBindValue::optional_text(intent.payload_ref.as_deref()),
            SqlBindValue::optional_text(intent.payload_hash.as_deref()),
            SqlBindValue::optional_text(intent.trace_id.as_deref()),
            SqlBindValue::Int64(0),
        ],
    )
}

pub fn initial_migration_sql() -> &'static str {
    r#"
CREATE TABLE iot_product (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    owner_type VARCHAR(32) NOT NULL,
    owner_id VARCHAR(128) NOT NULL,
    product_key VARCHAR(128) NOT NULL,
    display_name VARCHAR(200) NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_product_uuid UNIQUE (uuid),
    CONSTRAINT uk_iot_product_tenant_key UNIQUE (tenant_id, product_key)
);

CREATE TABLE iot_hardware_profile (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    owner_type VARCHAR(32) NOT NULL,
    owner_id VARCHAR(128) NOT NULL,
    profile_key VARCHAR(128) NOT NULL,
    chip_family VARCHAR(64) NOT NULL,
    runtime_profile VARCHAR(64) NOT NULL,
    connectivity_profile VARCHAR(64) NOT NULL,
    security_profile VARCHAR(64),
    ota_profile VARCHAR(64),
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_hardware_profile_uuid UNIQUE (uuid),
    CONSTRAINT uk_iot_hardware_profile_tenant_key UNIQUE (tenant_id, profile_key)
);

CREATE INDEX idx_iot_hardware_profile_tenant_chip
    ON iot_hardware_profile (tenant_id, chip_family, runtime_profile);

CREATE TABLE iot_protocol_profile (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    owner_type VARCHAR(32) NOT NULL,
    owner_id VARCHAR(128) NOT NULL,
    profile_key VARCHAR(128) NOT NULL,
    default_protocol_id VARCHAR(128) NOT NULL,
    allowed_transports TEXT NOT NULL,
    allowed_message_classes TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_protocol_profile_uuid UNIQUE (uuid),
    CONSTRAINT uk_iot_protocol_profile_tenant_key UNIQUE (tenant_id, profile_key)
);

CREATE INDEX idx_iot_protocol_profile_tenant_protocol
    ON iot_protocol_profile (tenant_id, default_protocol_id, status);

CREATE TABLE iot_capability_model (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    owner_type VARCHAR(32) NOT NULL,
    owner_id VARCHAR(128) NOT NULL,
    model_key VARCHAR(128) NOT NULL,
    display_name VARCHAR(200) NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_capability_model_tenant_key UNIQUE (tenant_id, model_key)
);

CREATE TABLE iot_capability_definition (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    capability_model_id BIGINT NOT NULL,
    capability_name VARCHAR(128) NOT NULL,
    capability_kind VARCHAR(32) NOT NULL,
    schema_json TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_capability_definition_tenant_model_name
        UNIQUE (tenant_id, capability_model_id, capability_name)
);

CREATE TABLE iot_device (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    owner_type VARCHAR(32) NOT NULL,
    owner_id VARCHAR(128) NOT NULL,
    device_key VARCHAR(128) NOT NULL,
    product_id BIGINT NOT NULL,
    hardware_profile_id BIGINT,
    protocol_profile_id BIGINT,
    display_name VARCHAR(200) NOT NULL,
    device_id VARCHAR(128) NOT NULL,
    client_id VARCHAR(128),
    serial_number VARCHAR(128),
    mac_address VARCHAR(128),
    chip_family VARCHAR(64),
    runtime_profile VARCHAR(64),
    firmware_version VARCHAR(64),
    auth_state INTEGER NOT NULL DEFAULT 0,
    lifecycle_state INTEGER NOT NULL DEFAULT 0,
    last_seen_at TIMESTAMP,
    metadata TEXT,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    deleted_at TIMESTAMP,
    deleted_by BIGINT,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_device_uuid UNIQUE (uuid),
    CONSTRAINT uk_iot_device_tenant_device_key UNIQUE (tenant_id, device_key),
    CONSTRAINT uk_iot_device_tenant_product_device_id UNIQUE (tenant_id, product_id, device_id),
    CONSTRAINT uk_iot_device_tenant_client_id UNIQUE (tenant_id, client_id)
);

CREATE INDEX idx_iot_device_tenant_product_status
    ON iot_device (tenant_id, product_id, status);

CREATE INDEX idx_iot_device_tenant_last_seen
    ON iot_device (tenant_id, last_seen_at);

CREATE TABLE iot_device_credential (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    device_id VARCHAR(128) NOT NULL,
    credential_type VARCHAR(64) NOT NULL,
    credential_hash VARCHAR(256),
    credential_ref VARCHAR(512),
    expires_at TIMESTAMP,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_device_credential_tenant_device_status
    ON iot_device_credential (tenant_id, device_id, status);

CREATE TABLE iot_device_binding (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    device_id VARCHAR(128) NOT NULL,
    binding_type VARCHAR(64) NOT NULL,
    target_type VARCHAR(64) NOT NULL,
    target_id VARCHAR(128) NOT NULL,
    role VARCHAR(64),
    status INTEGER NOT NULL,
    bound_at TIMESTAMP NOT NULL,
    bound_by BIGINT,
    expires_at TIMESTAMP,
    metadata TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_device_binding_tenant_target
    ON iot_device_binding (tenant_id, target_type, target_id, status);

CREATE TABLE iot_gateway_child_device (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    gateway_device_id VARCHAR(128) NOT NULL,
    child_device_id VARCHAR(128) NOT NULL,
    topology_role VARCHAR(64) NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_gateway_child_device_tenant_pair
        UNIQUE (tenant_id, gateway_device_id, child_device_id)
);

CREATE TABLE iot_device_connection (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    connection_id VARCHAR(128) NOT NULL,
    device_id VARCHAR(128),
    transport VARCHAR(64) NOT NULL,
    remote_addr VARCHAR(256),
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_device_connection_tenant_connection UNIQUE (tenant_id, connection_id)
);

CREATE INDEX idx_iot_device_connection_tenant_device_created
    ON iot_device_connection (tenant_id, device_id, created_at);

CREATE TABLE iot_device_session (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    device_id VARCHAR(128) NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    connection_id VARCHAR(128) NOT NULL,
    protocol_id VARCHAR(128) NOT NULL,
    adapter_id VARCHAR(128) NOT NULL,
    node_id VARCHAR(128),
    status INTEGER NOT NULL,
    connected_at TIMESTAMP NOT NULL,
    last_seen_at TIMESTAMP,
    disconnected_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_device_session_tenant_session UNIQUE (tenant_id, session_id)
);

CREATE INDEX idx_iot_device_session_tenant_device_status
    ON iot_device_session (tenant_id, device_id, status);

CREATE TABLE iot_device_online_lease (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    device_id VARCHAR(128) NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    node_id VARCHAR(128) NOT NULL,
    lease_expires_at TIMESTAMP NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_device_online_lease_tenant_device UNIQUE (tenant_id, device_id)
);

CREATE INDEX idx_iot_device_online_lease_tenant_expires
    ON iot_device_online_lease (tenant_id, lease_expires_at);

CREATE TABLE iot_command (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    command_id VARCHAR(128) NOT NULL,
    device_id VARCHAR(128) NOT NULL,
    session_id VARCHAR(128),
    capability_name VARCHAR(128) NOT NULL,
    command_name VARCHAR(128) NOT NULL,
    request_payload TEXT NOT NULL,
    request_media_resource_id VARCHAR(128),
    request_object_blob_id VARCHAR(128),
    request_media_resource_snapshot TEXT,
    status INTEGER NOT NULL,
    idempotency_key VARCHAR(128),
    timeout_at TIMESTAMP,
    ack_at TIMESTAMP,
    result_at TIMESTAMP,
    trace_id VARCHAR(128),
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_command_tenant_command_id UNIQUE (tenant_id, command_id),
    CONSTRAINT uk_iot_command_tenant_idempotency_key
        UNIQUE (tenant_id, organization_id, idempotency_key)
);

CREATE INDEX idx_iot_command_tenant_device_status_created
    ON iot_command (tenant_id, device_id, status, created_at);

CREATE INDEX idx_iot_command_tenant_status_timeout
    ON iot_command (tenant_id, status, timeout_at);

CREATE TABLE iot_command_delivery (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    command_id VARCHAR(128) NOT NULL,
    session_id VARCHAR(128),
    delivery_state VARCHAR(64) NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_command_delivery_tenant_session_status
    ON iot_command_delivery (tenant_id, session_id, status);

CREATE TABLE iot_command_result (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    command_id VARCHAR(128) NOT NULL,
    result_payload TEXT,
    result_media_resource_id VARCHAR(128),
    result_object_blob_id VARCHAR(128),
    result_media_resource_snapshot TEXT,
    result_code VARCHAR(128),
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_command_result_tenant_command
    ON iot_command_result (tenant_id, command_id);

CREATE TABLE iot_device_twin (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    device_id VARCHAR(128) NOT NULL,
    desired_version BIGINT NOT NULL DEFAULT 0,
    reported_version BIGINT NOT NULL DEFAULT 0,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_device_twin_tenant_device UNIQUE (tenant_id, device_id)
);

CREATE TABLE iot_device_twin_property (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    device_id VARCHAR(128) NOT NULL,
    property_name VARCHAR(128) NOT NULL,
    desired_value TEXT,
    desired_version BIGINT NOT NULL DEFAULT 0,
    desired_updated_at TIMESTAMP,
    reported_value TEXT,
    reported_version BIGINT NOT NULL DEFAULT 0,
    reported_updated_at TIMESTAMP,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_twin_property_tenant_device_property
        UNIQUE (tenant_id, device_id, property_name)
);

CREATE INDEX idx_iot_twin_property_tenant_device_property
    ON iot_device_twin_property (tenant_id, device_id, property_name);

CREATE TABLE iot_telemetry_latest (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    device_id VARCHAR(128) NOT NULL,
    metric_key VARCHAR(128) NOT NULL,
    metric_value TEXT NOT NULL,
    metric_type VARCHAR(32) NOT NULL,
    measured_at TIMESTAMP NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_telemetry_latest_tenant_device_key
        UNIQUE (tenant_id, device_id, metric_key)
);

CREATE INDEX idx_iot_telemetry_latest_tenant_device_key
    ON iot_telemetry_latest (tenant_id, device_id, metric_key);

CREATE TABLE iot_telemetry_event (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    device_id VARCHAR(128) NOT NULL,
    metric_key VARCHAR(128) NOT NULL,
    metric_value TEXT NOT NULL,
    measured_at TIMESTAMP NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_telemetry_event_tenant_device_time
    ON iot_telemetry_event (tenant_id, device_id, measured_at);

CREATE TABLE iot_device_event (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    device_id VARCHAR(128) NOT NULL,
    event_type VARCHAR(128) NOT NULL,
    event_payload TEXT NOT NULL,
    media_resource_id VARCHAR(128),
    object_blob_id VARCHAR(128),
    media_resource_snapshot TEXT,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_device_event_tenant_device_time
    ON iot_device_event (tenant_id, device_id, created_at);

CREATE TABLE iot_security_event (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    security_event_type VARCHAR(128) NOT NULL,
    severity VARCHAR(64) NOT NULL,
    actor_type VARCHAR(64),
    actor_id VARCHAR(128),
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    trace_id VARCHAR(128),
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_security_event_tenant_time
    ON iot_security_event (tenant_id, created_at);

CREATE TABLE iot_media_resource (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    owner_type VARCHAR(32) NOT NULL,
    owner_id VARCHAR(128) NOT NULL,
    media_resource_id VARCHAR(128) NOT NULL,
    kind VARCHAR(32) NOT NULL,
    source VARCHAR(32) NOT NULL,
    object_blob_id VARCHAR(128),
    resource_snapshot TEXT,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_media_resource_tenant_resource_id
        UNIQUE (tenant_id, media_resource_id)
);

CREATE INDEX idx_iot_media_resource_tenant_owner
    ON iot_media_resource (tenant_id, owner_type, owner_id, status);

CREATE INDEX idx_iot_media_resource_tenant_object_blob
    ON iot_media_resource (tenant_id, object_blob_id);

CREATE TABLE iot_device_media (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    owner_type VARCHAR(32) NOT NULL,
    owner_id VARCHAR(128) NOT NULL,
    media_role VARCHAR(64) NOT NULL,
    media_resource_id VARCHAR(128) NOT NULL,
    object_blob_id VARCHAR(128),
    resource_snapshot TEXT,
    alt_text VARCHAR(512),
    sort_order INTEGER NOT NULL DEFAULT 0,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_device_media_tenant_owner_role
    ON iot_device_media (tenant_id, owner_type, owner_id, media_role, sort_order);

CREATE INDEX idx_iot_device_media_tenant_media
    ON iot_device_media (tenant_id, media_resource_id);

CREATE TABLE iot_firmware_artifact (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    owner_type VARCHAR(32) NOT NULL,
    owner_id VARCHAR(128) NOT NULL,
    version_name VARCHAR(128) NOT NULL,
    media_resource_id VARCHAR(128) NOT NULL,
    object_blob_id VARCHAR(128),
    media_resource_snapshot TEXT,
    file_name VARCHAR(256),
    size_bytes BIGINT NOT NULL,
    sha256 VARCHAR(128) NOT NULL,
    signature TEXT,
    signature_algorithm VARCHAR(64),
    target_chip_family VARCHAR(64),
    target_runtime_profile VARCHAR(64),
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_firmware_artifact_tenant_media_resource
        UNIQUE (tenant_id, media_resource_id)
);

CREATE TABLE iot_firmware_rollout (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    owner_type VARCHAR(32) NOT NULL,
    owner_id VARCHAR(128) NOT NULL,
    artifact_id BIGINT NOT NULL,
    rollout_policy TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_by BIGINT,
    updated_by BIGINT,
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_firmware_rollout_tenant_status
    ON iot_firmware_rollout (tenant_id, status);

CREATE TABLE iot_firmware_rollout_target (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    rollout_id BIGINT NOT NULL,
    target_type VARCHAR(64) NOT NULL,
    target_id VARCHAR(128) NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_firmware_rollout_target_tenant_rollout
    ON iot_firmware_rollout_target (tenant_id, rollout_id);

CREATE TABLE iot_firmware_deployment (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    rollout_id BIGINT,
    device_id VARCHAR(128) NOT NULL,
    deployment_state VARCHAR(64) NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_firmware_deployment_tenant_device_status
    ON iot_firmware_deployment (tenant_id, device_id, status);

CREATE TABLE iot_provisioning_challenge (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    challenge_id VARCHAR(128) NOT NULL,
    device_hint VARCHAR(128),
    expires_at TIMESTAMP NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_provisioning_challenge_tenant_id UNIQUE (tenant_id, challenge_id)
);

CREATE INDEX idx_iot_provisioning_challenge_tenant_expires
    ON iot_provisioning_challenge (tenant_id, expires_at);

CREATE TABLE iot_activation_record (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    activation_id VARCHAR(128) NOT NULL,
    device_id VARCHAR(128),
    activation_profile VARCHAR(128) NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_activation_record_tenant_device
    ON iot_activation_record (tenant_id, device_id);

CREATE TABLE iot_protocol_message_dead_letter (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    protocol_id VARCHAR(128) NOT NULL,
    adapter_id VARCHAR(128) NOT NULL,
    device_id VARCHAR(128),
    reason_code VARCHAR(128) NOT NULL,
    payload_ref VARCHAR(512),
    payload_hash VARCHAR(128),
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    trace_id VARCHAR(128),
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_protocol_dead_letter_tenant_created
    ON iot_protocol_message_dead_letter (tenant_id, created_at);

CREATE TABLE iot_outbox_event (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    event_id VARCHAR(128) NOT NULL,
    event_type VARCHAR(128) NOT NULL,
    event_version VARCHAR(16) NOT NULL DEFAULT '1',
    aggregate_type VARCHAR(128) NOT NULL,
    aggregate_id VARCHAR(128) NOT NULL,
    payload TEXT NOT NULL,
    payload_hash VARCHAR(128),
    status INTEGER NOT NULL,
    next_attempt_at TIMESTAMP,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL,
    published_at TIMESTAMP,
    trace_id VARCHAR(128),
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_outbox_event_tenant_event_id UNIQUE (tenant_id, event_id)
);

CREATE INDEX idx_iot_outbox_event_status_next_attempt
    ON iot_outbox_event (status, next_attempt_at);

CREATE TABLE iot_inbox_event (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    source_system VARCHAR(128) NOT NULL,
    message_id VARCHAR(128) NOT NULL,
    consumer_name VARCHAR(128) NOT NULL,
    payload_hash VARCHAR(128),
    error_message VARCHAR(1000),
    processed_at TIMESTAMP,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_inbox_event_consumer_message
        UNIQUE (source_system, message_id, consumer_name)
);

CREATE TABLE iot_audit_log (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    operator_id BIGINT,
    action VARCHAR(128) NOT NULL,
    target_type VARCHAR(128) NOT NULL,
    target_id VARCHAR(128) NOT NULL,
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    trace_id VARCHAR(128),
    PRIMARY KEY (id)
);

CREATE INDEX idx_iot_audit_log_tenant_created
    ON iot_audit_log (tenant_id, created_at);

CREATE TABLE iot_protocol_ingest_record (
    id BIGINT NOT NULL,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    data_scope INTEGER NOT NULL DEFAULT 0,
    protocol_id VARCHAR(128) NOT NULL,
    adapter_id VARCHAR(128) NOT NULL,
    device_id VARCHAR(128),
    message_id VARCHAR(128),
    correlation_id VARCHAR(128),
    media_resource_id VARCHAR(128),
    object_blob_id VARCHAR(128),
    media_resource_snapshot TEXT,
    idempotency_key VARCHAR(256) NOT NULL,
    trace_id VARCHAR(128),
    status INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_iot_protocol_ingest_tenant_idempotency
        UNIQUE (tenant_id, idempotency_key)
);

CREATE INDEX idx_iot_protocol_ingest_tenant_message
    ON iot_protocol_ingest_record (tenant_id, protocol_id, message_id);
"#
}
