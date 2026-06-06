use sdkwork_aiot_protocol::{
    standard_protocol_catalog, CapabilityBridge, CodecKind, HandshakeContext, MessageClass,
    ProtocolAdapterManifest, ProtocolEnvelope, ProtocolPluginScope, SessionPolicy,
    TransportBinding,
};

#[test]
fn protocol_envelope_is_transport_neutral() {
    let envelope = ProtocolEnvelope::builder("xiaozhi.websocket", MessageClass::Handshake)
        .tenant("t1")
        .organization("o1")
        .device("dev1")
        .session("sess1")
        .semantic_type("hello")
        .json_payload(r#"{"type":"hello"}"#)
        .build();

    assert_eq!(envelope.protocol_id, "xiaozhi.websocket");
    assert_eq!(envelope.message_class, MessageClass::Handshake);
    assert_eq!(envelope.semantic_type, "hello");
    assert_eq!(envelope.content_type, "application/json");
}

#[test]
fn protocol_envelope_preserves_reliability_and_trace_identifiers() {
    let envelope = ProtocolEnvelope::builder("mqtt.v5", MessageClass::Telemetry)
        .message_id("msg-001")
        .correlation_id("corr-001")
        .idempotency_key("idem-001")
        .trace_id("trace-001")
        .device("device-001")
        .semantic_type("temperature")
        .json_payload(r#"{"temperature":21}"#)
        .build();

    assert_eq!(envelope.message_id.as_deref(), Some("msg-001"));
    assert_eq!(envelope.correlation_id.as_deref(), Some("corr-001"));
    assert_eq!(envelope.idempotency_key.as_deref(), Some("idem-001"));
    assert_eq!(envelope.trace_id.as_deref(), Some("trace-001"));
}

#[test]
fn protocol_envelope_preserves_media_resource_identity_fields() {
    let envelope = ProtocolEnvelope::builder("xiaozhi.websocket", MessageClass::MediaFrame)
        .device("device-001")
        .semantic_type("audio")
        .media_resource_id("media-res-001")
        .object_blob_id("obj-blob-001")
        .media_resource_snapshot(
            r#"{"id":"media-res-001","kind":"audio","source":"object_storage"}"#,
        )
        .build();

    assert_eq!(envelope.media_resource_id.as_deref(), Some("media-res-001"));
    assert_eq!(envelope.object_blob_id.as_deref(), Some("obj-blob-001"));
    assert!(envelope
        .media_resource_snapshot
        .as_deref()
        .unwrap_or_default()
        .contains(r#""kind":"audio""#));
}

#[test]
fn adapter_manifest_declares_protocols_transports_and_capabilities() {
    let manifest = ProtocolAdapterManifest::new("xiaozhi", "0.1.0")
        .with_scope(ProtocolPluginScope::CompatibilityPlugin)
        .with_protocol("xiaozhi.websocket")
        .with_transport(TransportBinding::WebSocket)
        .with_codec(CodecKind::JsonText)
        .with_codec(CodecKind::BinaryMedia)
        .with_session_policy(SessionPolicy::StatefulDeviceSession)
        .with_capability_bridge("mcp_jsonrpc")
        .with_security_mode("bearer_token")
        .with_hardware_family("esp32_s3")
        .with_runtime_profile("esp_idf")
        .with_firmware_profile("xiaozhi_ota");

    assert_eq!(manifest.plugin_id, "xiaozhi");
    assert_eq!(manifest.scope, ProtocolPluginScope::CompatibilityPlugin);
    assert!(manifest
        .protocol_ids
        .contains(&"xiaozhi.websocket".to_string()));
    assert!(manifest.transports.contains(&TransportBinding::WebSocket));
    assert!(manifest.codecs.contains(&CodecKind::JsonText));
    assert!(manifest.codecs.contains(&CodecKind::BinaryMedia));
    assert!(manifest
        .session_policies
        .contains(&SessionPolicy::StatefulDeviceSession));
    assert!(manifest
        .capability_bridges
        .contains(&"mcp_jsonrpc".to_string()));
    assert!(manifest.hardware_families.contains(&"esp32_s3".to_string()));
    assert!(manifest.runtime_profiles.contains(&"esp_idf".to_string()));
    assert!(manifest
        .firmware_profiles
        .contains(&"xiaozhi_ota".to_string()));
}

#[test]
fn adapter_manifest_can_describe_bridge_and_stateless_protocol_plugins() {
    let manifest = ProtocolAdapterManifest::new("lorawan-chirpstack", "0.1.0")
        .with_scope(ProtocolPluginScope::BridgeAdapter)
        .with_protocol("lorawan.chirpstack")
        .with_transport(TransportBinding::Mqtt)
        .with_transport(TransportBinding::Http)
        .with_codec(CodecKind::JsonText)
        .with_codec(CodecKind::BinaryPayload)
        .with_session_policy(SessionPolicy::StatelessUplink)
        .with_capability_bridge("lorawan_payload_codec")
        .with_hardware_family("stm32wl")
        .with_hardware_family("sx126x")
        .with_runtime_profile("zephyr");

    assert_eq!(manifest.scope, ProtocolPluginScope::BridgeAdapter);
    assert!(manifest.transports.contains(&TransportBinding::Mqtt));
    assert!(manifest.codecs.contains(&CodecKind::BinaryPayload));
    assert!(manifest
        .session_policies
        .contains(&SessionPolicy::StatelessUplink));
    assert!(manifest.hardware_families.contains(&"stm32wl".to_string()));
    assert!(manifest.runtime_profiles.contains(&"zephyr".to_string()));
}

#[test]
fn handshake_context_keeps_headers_out_of_core_domain() {
    let ctx = HandshakeContext::new(TransportBinding::WebSocket)
        .with_header("Protocol-Version", "3")
        .with_header("Device-Id", "aa:bb")
        .with_path("/iot/xiaozhi/ws");

    assert_eq!(ctx.transport, TransportBinding::WebSocket);
    assert_eq!(ctx.header("Protocol-Version"), Some("3"));
    assert_eq!(ctx.path.as_deref(), Some("/iot/xiaozhi/ws"));
}

#[test]
fn standard_protocol_catalog_covers_major_iot_ecosystems_without_core_coupling() {
    let catalog = standard_protocol_catalog();

    for expected in [
        "xiaozhi.websocket",
        "mqtt.v3_1_1",
        "mqtt.v5",
        "coap.lwm2m",
        "matter.bridge",
        "zigbee2mqtt.bridge",
        "lorawan.chirpstack",
        "modbus.bridge",
        "opcua.bridge",
        "esphome.native",
        "tasmota.mqtt",
        "wled.mqtt",
        "openbeken.mqtt",
        "raspberrypi.linux_gateway",
        "raspberrypi.pico_mqtt",
    ] {
        assert!(
            catalog
                .iter()
                .any(|protocol| protocol.protocol_id == expected),
            "missing protocol {expected}"
        );
    }

    let xiaozhi = catalog
        .iter()
        .find(|protocol| protocol.protocol_id == "xiaozhi.websocket")
        .expect("xiaozhi protocol");
    assert_eq!(xiaozhi.scope, ProtocolPluginScope::CompatibilityPlugin);
    assert!(xiaozhi.transports.contains(&TransportBinding::WebSocket));
    assert!(xiaozhi
        .capability_bridges
        .contains(&CapabilityBridge::McpJsonRpc));
}

#[test]
fn protocol_catalog_distinguishes_native_protocols_from_bridge_integrations() {
    let catalog = standard_protocol_catalog();
    let mqtt = catalog
        .iter()
        .find(|protocol| protocol.protocol_id == "mqtt.v5")
        .expect("mqtt");
    let modbus = catalog
        .iter()
        .find(|protocol| protocol.protocol_id == "modbus.bridge")
        .expect("modbus");
    let matter = catalog
        .iter()
        .find(|protocol| protocol.protocol_id == "matter.bridge")
        .expect("matter");

    assert_eq!(mqtt.scope, ProtocolPluginScope::StandardAdapter);
    assert_eq!(modbus.scope, ProtocolPluginScope::BridgeAdapter);
    assert_eq!(matter.scope, ProtocolPluginScope::BridgeAdapter);
    assert!(modbus
        .capability_bridges
        .contains(&CapabilityBridge::RegisterMap));
    assert!(matter
        .capability_bridges
        .contains(&CapabilityBridge::MatterCluster));
}

#[test]
fn mqtt_standard_adapters_use_rmqtt_as_the_single_broker_reference() {
    let catalog = standard_protocol_catalog();

    for protocol_id in ["mqtt.v3_1_1", "mqtt.v5"] {
        let mqtt = catalog
            .iter()
            .find(|protocol| protocol.protocol_id == protocol_id)
            .unwrap_or_else(|| panic!("missing {protocol_id}"));

        assert_eq!(
            mqtt.reference_projects,
            vec!["rmqtt"],
            "{protocol_id} must use rmqtt as the only MQTT broker/server implementation reference"
        );
        assert!(mqtt.transports.contains(&TransportBinding::Mqtt));
        assert!(mqtt
            .capability_bridges
            .contains(&CapabilityBridge::MqttTopic));
    }
}

#[test]
fn protocol_catalog_reference_projects_are_curated_external_baselines() {
    let catalog = standard_protocol_catalog();
    let allowed = [
        "xiaozhi-esp32",
        "rmqtt",
        "esphome",
        "tasmota",
        "zigbee2mqtt",
        "wled",
    ];

    for protocol in catalog {
        for reference_project in protocol.reference_projects {
            assert!(
                allowed.contains(&reference_project),
                "{} references non-curated external project {reference_project}",
                protocol.protocol_id
            );
        }
    }
}

#[test]
fn raspberry_pi_protocols_model_linux_gateway_and_pico_mcu_separately() {
    let catalog = standard_protocol_catalog();
    let linux_gateway = catalog
        .iter()
        .find(|protocol| protocol.protocol_id == "raspberrypi.linux_gateway")
        .expect("raspberrypi linux gateway");
    let pico = catalog
        .iter()
        .find(|protocol| protocol.protocol_id == "raspberrypi.pico_mqtt")
        .expect("raspberrypi pico");

    assert_eq!(linux_gateway.scope, ProtocolPluginScope::BridgeAdapter);
    assert!(linux_gateway.transports.contains(&TransportBinding::Mqtt));
    assert!(linux_gateway.transports.contains(&TransportBinding::Http));
    assert!(linux_gateway
        .capability_bridges
        .contains(&CapabilityBridge::StandardCapability));
    assert!(linux_gateway.reference_projects.is_empty());

    assert_eq!(pico.scope, ProtocolPluginScope::CompatibilityPlugin);
    assert!(pico.transports.contains(&TransportBinding::Mqtt));
    assert!(pico
        .capability_bridges
        .contains(&CapabilityBridge::MqttTopic));
    assert!(pico.reference_projects.is_empty());
}
