use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportBinding {
    Tcp,
    Udp,
    WebSocket,
    Http,
    Mqtt,
    Coap,
    Serial,
    Ble,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolPluginScope {
    StandardAdapter,
    CompatibilityPlugin,
    BridgeAdapter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecKind {
    JsonText,
    BinaryMedia,
    BinaryPayload,
    JsonRpc,
    Protobuf,
    Cbor,
    TopicPayload,
    RegisterMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPolicy {
    StatefulDeviceSession,
    StatelessUplink,
    BrokerSession,
    BridgeSession,
    GatewayMultiplexedSession,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityBridge {
    StandardCapability,
    McpJsonRpc,
    Lwm2mObject,
    MatterCluster,
    ZigbeeCluster,
    LorawanPayloadCodec,
    RegisterMap,
    OpcUaNode,
    MqttTopic,
    FirmwareOta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolCatalogEntry {
    pub protocol_id: &'static str,
    pub display_name: &'static str,
    pub scope: ProtocolPluginScope,
    pub transports: Vec<TransportBinding>,
    pub capability_bridges: Vec<CapabilityBridge>,
    pub reference_projects: Vec<&'static str>,
}

impl ProtocolCatalogEntry {
    pub fn new(
        protocol_id: &'static str,
        display_name: &'static str,
        scope: ProtocolPluginScope,
    ) -> Self {
        Self {
            protocol_id,
            display_name,
            scope,
            transports: Vec::new(),
            capability_bridges: Vec::new(),
            reference_projects: Vec::new(),
        }
    }

    pub fn with_transport(mut self, transport: TransportBinding) -> Self {
        self.transports.push(transport);
        self
    }

    pub fn with_capability_bridge(mut self, bridge: CapabilityBridge) -> Self {
        self.capability_bridges.push(bridge);
        self
    }

    pub fn with_reference_project(mut self, project: &'static str) -> Self {
        self.reference_projects.push(project);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageClass {
    Handshake,
    Auth,
    Heartbeat,
    Disconnect,
    Provisioning,
    Telemetry,
    Event,
    PropertyReport,
    PropertySet,
    TwinDesired,
    TwinReported,
    CommandRequest,
    CommandAck,
    CommandResult,
    MediaFrame,
    OtaCheck,
    OtaDeploy,
    GatewayTopology,
    SecurityEvent,
    Diagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolEnvelope {
    pub message_id: Option<String>,
    pub protocol_id: String,
    pub protocol_version: Option<String>,
    pub adapter_id: Option<String>,
    pub tenant_id: Option<String>,
    pub organization_id: Option<String>,
    pub product_id: Option<String>,
    pub device_id: Option<String>,
    pub client_id: Option<String>,
    pub connection_id: Option<String>,
    pub session_id: Option<String>,
    pub direction: Option<String>,
    pub message_class: MessageClass,
    pub semantic_type: String,
    pub content_type: String,
    pub payload_encoding: String,
    pub payload: Vec<u8>,
    pub qos: Option<String>,
    pub sequence_no: Option<u64>,
    pub timestamp_ms: Option<i64>,
    pub correlation_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub trace_id: Option<String>,
    pub extensions: BTreeMap<String, String>,
}

impl ProtocolEnvelope {
    pub fn builder(
        protocol_id: impl Into<String>,
        message_class: MessageClass,
    ) -> ProtocolEnvelopeBuilder {
        ProtocolEnvelopeBuilder {
            envelope: ProtocolEnvelope {
                message_id: None,
                protocol_id: protocol_id.into(),
                protocol_version: None,
                adapter_id: None,
                tenant_id: None,
                organization_id: None,
                product_id: None,
                device_id: None,
                client_id: None,
                connection_id: None,
                session_id: None,
                direction: None,
                message_class,
                semantic_type: String::new(),
                content_type: "application/octet-stream".to_string(),
                payload_encoding: "binary".to_string(),
                payload: Vec::new(),
                qos: None,
                sequence_no: None,
                timestamp_ms: None,
                correlation_id: None,
                idempotency_key: None,
                trace_id: None,
                extensions: BTreeMap::new(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolEnvelopeBuilder {
    envelope: ProtocolEnvelope,
}

impl ProtocolEnvelopeBuilder {
    pub fn message_id(mut self, message_id: impl Into<String>) -> Self {
        self.envelope.message_id = Some(message_id.into());
        self
    }

    pub fn protocol_version(mut self, protocol_version: impl Into<String>) -> Self {
        self.envelope.protocol_version = Some(protocol_version.into());
        self
    }

    pub fn adapter(mut self, adapter_id: impl Into<String>) -> Self {
        self.envelope.adapter_id = Some(adapter_id.into());
        self
    }

    pub fn tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.envelope.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn organization(mut self, organization_id: impl Into<String>) -> Self {
        self.envelope.organization_id = Some(organization_id.into());
        self
    }

    pub fn device(mut self, device_id: impl Into<String>) -> Self {
        self.envelope.device_id = Some(device_id.into());
        self
    }

    pub fn client(mut self, client_id: impl Into<String>) -> Self {
        self.envelope.client_id = Some(client_id.into());
        self
    }

    pub fn session(mut self, session_id: impl Into<String>) -> Self {
        self.envelope.session_id = Some(session_id.into());
        self
    }

    pub fn correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.envelope.correlation_id = Some(correlation_id.into());
        self
    }

    pub fn idempotency_key(mut self, idempotency_key: impl Into<String>) -> Self {
        self.envelope.idempotency_key = Some(idempotency_key.into());
        self
    }

    pub fn trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.envelope.trace_id = Some(trace_id.into());
        self
    }

    pub fn semantic_type(mut self, semantic_type: impl Into<String>) -> Self {
        self.envelope.semantic_type = semantic_type.into();
        self
    }

    pub fn json_payload(mut self, payload: impl AsRef<[u8]>) -> Self {
        self.envelope.content_type = "application/json".to_string();
        self.envelope.payload_encoding = "utf8".to_string();
        self.envelope.payload = payload.as_ref().to_vec();
        self
    }

    pub fn binary_payload(mut self, payload: impl AsRef<[u8]>) -> Self {
        self.envelope.content_type = "application/octet-stream".to_string();
        self.envelope.payload_encoding = "binary".to_string();
        self.envelope.payload = payload.as_ref().to_vec();
        self
    }

    pub fn extension(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.envelope.extensions.insert(name.into(), value.into());
        self
    }

    pub fn build(self) -> ProtocolEnvelope {
        self.envelope
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolAdapterManifest {
    pub plugin_id: String,
    pub plugin_version: String,
    pub scope: ProtocolPluginScope,
    pub protocol_ids: Vec<String>,
    pub transports: Vec<TransportBinding>,
    pub codecs: Vec<CodecKind>,
    pub session_policies: Vec<SessionPolicy>,
    pub capability_bridges: Vec<String>,
    pub security_modes: Vec<String>,
    pub ota_profiles: Vec<String>,
    pub provisioning_profiles: Vec<String>,
    pub hardware_families: Vec<String>,
    pub runtime_profiles: Vec<String>,
    pub firmware_profiles: Vec<String>,
}

impl ProtocolAdapterManifest {
    pub fn new(plugin_id: impl Into<String>, plugin_version: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            plugin_version: plugin_version.into(),
            scope: ProtocolPluginScope::StandardAdapter,
            protocol_ids: Vec::new(),
            transports: Vec::new(),
            codecs: Vec::new(),
            session_policies: Vec::new(),
            capability_bridges: Vec::new(),
            security_modes: Vec::new(),
            ota_profiles: Vec::new(),
            provisioning_profiles: Vec::new(),
            hardware_families: Vec::new(),
            runtime_profiles: Vec::new(),
            firmware_profiles: Vec::new(),
        }
    }

    pub fn with_scope(mut self, scope: ProtocolPluginScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn with_protocol(mut self, protocol_id: impl Into<String>) -> Self {
        self.protocol_ids.push(protocol_id.into());
        self
    }

    pub fn with_transport(mut self, transport: TransportBinding) -> Self {
        self.transports.push(transport);
        self
    }

    pub fn with_codec(mut self, codec: CodecKind) -> Self {
        self.codecs.push(codec);
        self
    }

    pub fn with_session_policy(mut self, policy: SessionPolicy) -> Self {
        self.session_policies.push(policy);
        self
    }

    pub fn with_capability_bridge(mut self, bridge: impl Into<String>) -> Self {
        self.capability_bridges.push(bridge.into());
        self
    }

    pub fn with_security_mode(mut self, mode: impl Into<String>) -> Self {
        self.security_modes.push(mode.into());
        self
    }

    pub fn with_ota_profile(mut self, profile: impl Into<String>) -> Self {
        self.ota_profiles.push(profile.into());
        self
    }

    pub fn with_provisioning_profile(mut self, profile: impl Into<String>) -> Self {
        self.provisioning_profiles.push(profile.into());
        self
    }

    pub fn with_hardware_family(mut self, family: impl Into<String>) -> Self {
        self.hardware_families.push(family.into());
        self
    }

    pub fn with_runtime_profile(mut self, profile: impl Into<String>) -> Self {
        self.runtime_profiles.push(profile.into());
        self
    }

    pub fn with_firmware_profile(mut self, profile: impl Into<String>) -> Self {
        self.firmware_profiles.push(profile.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakeContext {
    pub transport: TransportBinding,
    pub headers: BTreeMap<String, String>,
    pub path: Option<String>,
}

impl HandshakeContext {
    pub fn new(transport: TransportBinding) -> Self {
        Self {
            transport,
            headers: BTreeMap::new(),
            path: None,
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }
}

pub trait ProtocolAdapter {
    fn manifest(&self) -> ProtocolAdapterManifest;
}

pub trait MessageCodec {
    fn decode(&self, frame: InboundFrame) -> Result<ProtocolEnvelope, ProtocolError>;
    fn encode(&self, envelope: ProtocolEnvelope) -> Result<OutboundFrame, ProtocolError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundFrame {
    pub binary: bool,
    pub payload: Vec<u8>,
}

impl InboundFrame {
    pub fn text(text: impl AsRef<str>) -> Self {
        Self {
            binary: false,
            payload: text.as_ref().as_bytes().to_vec(),
        }
    }

    pub fn binary(payload: impl AsRef<[u8]>) -> Self {
        Self {
            binary: true,
            payload: payload.as_ref().to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundFrame {
    pub binary: bool,
    pub payload: Vec<u8>,
}

impl OutboundFrame {
    pub fn text(text: impl AsRef<str>) -> Self {
        Self {
            binary: false,
            payload: text.as_ref().as_bytes().to_vec(),
        }
    }

    pub fn binary(payload: impl AsRef<[u8]>) -> Self {
        Self {
            binary: true,
            payload: payload.as_ref().to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolError {
    pub code: String,
    pub message: String,
}

impl ProtocolError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

pub fn standard_protocol_catalog() -> Vec<ProtocolCatalogEntry> {
    vec![
        ProtocolCatalogEntry::new(
            "xiaozhi.websocket",
            "Xiaozhi WebSocket",
            ProtocolPluginScope::CompatibilityPlugin,
        )
        .with_transport(TransportBinding::WebSocket)
        .with_capability_bridge(CapabilityBridge::McpJsonRpc)
        .with_capability_bridge(CapabilityBridge::FirmwareOta)
        .with_reference_project("xiaozhi-esp32"),
        ProtocolCatalogEntry::new(
            "xiaozhi.mqtt_udp",
            "Xiaozhi MQTT and UDP media",
            ProtocolPluginScope::CompatibilityPlugin,
        )
        .with_transport(TransportBinding::Mqtt)
        .with_transport(TransportBinding::Udp)
        .with_capability_bridge(CapabilityBridge::McpJsonRpc)
        .with_reference_project("xiaozhi-esp32"),
        ProtocolCatalogEntry::new(
            "mqtt.v3_1_1",
            "MQTT 3.1.1",
            ProtocolPluginScope::StandardAdapter,
        )
        .with_transport(TransportBinding::Mqtt)
        .with_capability_bridge(CapabilityBridge::MqttTopic)
        .with_reference_project("rmqtt"),
        ProtocolCatalogEntry::new("mqtt.v5", "MQTT 5", ProtocolPluginScope::StandardAdapter)
            .with_transport(TransportBinding::Mqtt)
            .with_capability_bridge(CapabilityBridge::MqttTopic)
            .with_reference_project("rmqtt"),
        ProtocolCatalogEntry::new(
            "coap.lwm2m",
            "CoAP LwM2M",
            ProtocolPluginScope::BridgeAdapter,
        )
        .with_transport(TransportBinding::Coap)
        .with_capability_bridge(CapabilityBridge::Lwm2mObject),
        ProtocolCatalogEntry::new(
            "matter.bridge",
            "Matter Bridge",
            ProtocolPluginScope::BridgeAdapter,
        )
        .with_transport(TransportBinding::Tcp)
        .with_capability_bridge(CapabilityBridge::MatterCluster),
        ProtocolCatalogEntry::new(
            "zigbee2mqtt.bridge",
            "Zigbee2MQTT Bridge",
            ProtocolPluginScope::BridgeAdapter,
        )
        .with_transport(TransportBinding::Mqtt)
        .with_capability_bridge(CapabilityBridge::ZigbeeCluster)
        .with_reference_project("zigbee2mqtt"),
        ProtocolCatalogEntry::new(
            "lorawan.chirpstack",
            "LoRaWAN ChirpStack Bridge",
            ProtocolPluginScope::BridgeAdapter,
        )
        .with_transport(TransportBinding::Mqtt)
        .with_transport(TransportBinding::Http)
        .with_capability_bridge(CapabilityBridge::LorawanPayloadCodec),
        ProtocolCatalogEntry::new(
            "modbus.bridge",
            "Modbus Bridge",
            ProtocolPluginScope::BridgeAdapter,
        )
        .with_transport(TransportBinding::Tcp)
        .with_transport(TransportBinding::Serial)
        .with_capability_bridge(CapabilityBridge::RegisterMap),
        ProtocolCatalogEntry::new(
            "opcua.bridge",
            "OPC UA Bridge",
            ProtocolPluginScope::BridgeAdapter,
        )
        .with_transport(TransportBinding::Tcp)
        .with_capability_bridge(CapabilityBridge::OpcUaNode),
        ProtocolCatalogEntry::new(
            "esphome.native",
            "ESPHome Native API",
            ProtocolPluginScope::CompatibilityPlugin,
        )
        .with_transport(TransportBinding::Tcp)
        .with_capability_bridge(CapabilityBridge::StandardCapability)
        .with_reference_project("esphome"),
        ProtocolCatalogEntry::new(
            "tasmota.mqtt",
            "Tasmota MQTT",
            ProtocolPluginScope::CompatibilityPlugin,
        )
        .with_transport(TransportBinding::Mqtt)
        .with_capability_bridge(CapabilityBridge::MqttTopic)
        .with_reference_project("tasmota"),
        ProtocolCatalogEntry::new(
            "wled.mqtt",
            "WLED MQTT and JSON",
            ProtocolPluginScope::CompatibilityPlugin,
        )
        .with_transport(TransportBinding::Mqtt)
        .with_transport(TransportBinding::Http)
        .with_capability_bridge(CapabilityBridge::MqttTopic)
        .with_capability_bridge(CapabilityBridge::StandardCapability)
        .with_reference_project("wled"),
        ProtocolCatalogEntry::new(
            "raspberrypi.linux_gateway",
            "Raspberry Pi Linux Gateway",
            ProtocolPluginScope::BridgeAdapter,
        )
        .with_transport(TransportBinding::Mqtt)
        .with_transport(TransportBinding::Http)
        .with_transport(TransportBinding::WebSocket)
        .with_capability_bridge(CapabilityBridge::StandardCapability)
        .with_capability_bridge(CapabilityBridge::MqttTopic),
        ProtocolCatalogEntry::new(
            "raspberrypi.pico_mqtt",
            "Raspberry Pi Pico MQTT",
            ProtocolPluginScope::CompatibilityPlugin,
        )
        .with_transport(TransportBinding::Mqtt)
        .with_transport(TransportBinding::Http)
        .with_capability_bridge(CapabilityBridge::MqttTopic)
        .with_capability_bridge(CapabilityBridge::StandardCapability),
        ProtocolCatalogEntry::new(
            "openbeken.mqtt",
            "OpenBeken MQTT",
            ProtocolPluginScope::CompatibilityPlugin,
        )
        .with_transport(TransportBinding::Mqtt)
        .with_capability_bridge(CapabilityBridge::MqttTopic),
    ]
}
