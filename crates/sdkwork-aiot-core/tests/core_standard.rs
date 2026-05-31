use sdkwork_aiot_contract::AiotOwnershipRef;
use sdkwork_aiot_core::{
    protocol_ingest_plan, CapabilityDefinition, CapabilityKind, CommandStatus, Device,
    DeviceCommand, DomainEventKind, HardwareClass, HardwareProfile, Product, ProtocolIngestAction,
    ProtocolIngestRecord, ProtocolProfile,
};

#[test]
fn device_registry_keeps_identity_separate_from_sessions() {
    let product = Product::new("prod1", "Voice Device");
    let device = Device::new("dev1", "device-key-1", product.product_id.clone())
        .with_owner(AiotOwnershipRef::iam_user("u1"));

    assert_eq!(device.device_id, "dev1");
    assert_eq!(device.product_id, "prod1");
    assert_eq!(device.owner.owner_type, "user");
}

#[test]
fn command_lifecycle_is_explicit() {
    let command = DeviceCommand::new("cmd1", "dev1", "speaker", "setVolume")
        .mark_dispatched("sess1")
        .mark_acknowledged()
        .mark_succeeded(r#"{"ok":true}"#);

    assert_eq!(command.status, CommandStatus::Succeeded);
    assert_eq!(command.session_id.as_deref(), Some("sess1"));
    assert_eq!(command.result_payload.as_deref(), Some(r#"{"ok":true}"#));
}

#[test]
fn hardware_and_protocol_profiles_abstract_chip_and_firmware_ecosystems() {
    let profile = HardwareProfile::new("hw-esp32-s3", "esp32_s3")
        .with_hardware_class(HardwareClass::Mcu)
        .with_runtime("esp_idf")
        .with_runtime("freertos")
        .with_connectivity("wifi")
        .with_ota_profile("xiaozhi_ota");

    assert_eq!(profile.chip_family, "esp32_s3");
    assert_eq!(profile.hardware_class, HardwareClass::Mcu);
    assert!(profile.runtime_profiles.contains(&"esp_idf".to_string()));
    assert!(profile.connectivity_profiles.contains(&"wifi".to_string()));
    assert!(profile.ota_profiles.contains(&"xiaozhi_ota".to_string()));

    let protocol = ProtocolProfile::new("proto-xiaozhi", "xiaozhi.websocket")
        .allow_transport("websocket")
        .allow_message_class("handshake")
        .allow_message_class("media_frame");

    assert_eq!(protocol.default_protocol_id, "xiaozhi.websocket");
    assert!(protocol
        .allowed_transports
        .contains(&"websocket".to_string()));
    assert!(protocol
        .allowed_message_classes
        .contains(&"media_frame".to_string()));
}

#[test]
fn hardware_profiles_distinguish_linux_gateways_from_mcu_firmware() {
    let raspberry_pi_gateway = HardwareProfile::new("hw-raspberry-pi-5", "bcm2712")
        .with_hardware_class(HardwareClass::LinuxSbc)
        .with_hardware_class(HardwareClass::EdgeGateway)
        .with_runtime("linux")
        .with_runtime("docker")
        .with_runtime("home_assistant")
        .with_connectivity("ethernet")
        .with_connectivity("wifi")
        .with_connectivity("zigbee_usb")
        .with_security_profile("tpm")
        .with_ota_profile("apt_container_image");

    assert_eq!(raspberry_pi_gateway.chip_family, "bcm2712");
    assert_eq!(raspberry_pi_gateway.hardware_class, HardwareClass::LinuxSbc);
    assert!(raspberry_pi_gateway
        .hardware_classes
        .contains(&HardwareClass::EdgeGateway));
    assert!(raspberry_pi_gateway
        .runtime_profiles
        .contains(&"home_assistant".to_string()));
    assert!(raspberry_pi_gateway
        .connectivity_profiles
        .contains(&"zigbee_usb".to_string()));

    let pico = HardwareProfile::new("hw-raspberry-pi-pico-w", "rp2040")
        .with_hardware_class(HardwareClass::Mcu)
        .with_runtime("pico_sdk")
        .with_runtime("micropython")
        .with_runtime("zephyr")
        .with_connectivity("wifi")
        .with_security_profile("device_secret")
        .with_ota_profile("http_firmware");

    assert_eq!(pico.hardware_class, HardwareClass::Mcu);
    assert!(pico.runtime_profiles.contains(&"pico_sdk".to_string()));
    assert!(!pico.runtime_profiles.contains(&"docker".to_string()));
}

#[test]
fn capability_model_is_semantic_not_transport_specific() {
    let capability = CapabilityDefinition::new("speaker.volume", CapabilityKind::Property)
        .with_command("setVolume")
        .with_event("volumeChanged")
        .with_protocol_mapping("xiaozhi.websocket", "audio.speaker.volume")
        .with_protocol_mapping("mqtt.v5", "devices/{deviceId}/commands/volume");

    assert_eq!(capability.name, "speaker.volume");
    assert_eq!(capability.kind, CapabilityKind::Property);
    assert!(capability.commands.contains(&"setVolume".to_string()));
    assert!(capability.events.contains(&"volumeChanged".to_string()));
    assert_eq!(
        capability.protocol_mappings.get("xiaozhi.websocket"),
        Some(&"audio.speaker.volume".to_string())
    );
}

#[test]
fn protocol_ingest_records_are_transport_neutral_domain_inputs() {
    let record = ProtocolIngestRecord::new(
        "xiaozhi.websocket",
        "xiaozhi",
        "device-001",
        ProtocolIngestAction::OpenSession,
        "device_session",
    )
    .with_client_id("client-abc")
    .with_session_id("session-001")
    .with_trace_id("trace-001");

    assert_eq!(record.protocol_id, "xiaozhi.websocket");
    assert_eq!(record.plugin_id, "xiaozhi");
    assert_eq!(record.device_id, "device-001");
    assert_eq!(record.pipeline, "device_session");
    assert_eq!(record.action, ProtocolIngestAction::OpenSession);
    assert_eq!(record.client_id.as_deref(), Some("client-abc"));
    assert_eq!(record.session_id.as_deref(), Some("session-001"));
    assert_eq!(record.trace_id.as_deref(), Some("trace-001"));
}

#[test]
fn protocol_ingest_plan_maps_actions_to_domain_events_and_persistence_targets() {
    for (action, expected_event, expected_table) in [
        (
            ProtocolIngestAction::OpenSession,
            DomainEventKind::DeviceSessionOpened,
            "iot_device_session",
        ),
        (
            ProtocolIngestAction::RecordTelemetry,
            DomainEventKind::TelemetryRecorded,
            "iot_telemetry_event",
        ),
        (
            ProtocolIngestAction::DispatchCommand,
            DomainEventKind::CommandDispatchRequested,
            "iot_command_delivery",
        ),
        (
            ProtocolIngestAction::ProcessMediaFrame,
            DomainEventKind::MediaFrameReceived,
            "iot_device_event",
        ),
        (
            ProtocolIngestAction::EvaluateOta,
            DomainEventKind::OtaCheckRequested,
            "iot_firmware_deployment",
        ),
    ] {
        let record =
            ProtocolIngestRecord::new("xiaozhi.websocket", "xiaozhi", "device-001", action, "test");
        let plan = protocol_ingest_plan(&record);

        assert_eq!(plan.event_kind, expected_event);
        assert_eq!(plan.primary_table, expected_table);
        assert_eq!(plan.outbox_topic, "iot.protocol.ingested");
        assert!(plan.emit_outbox_event);
    }
}

#[test]
fn domain_event_kinds_have_stable_dotted_event_names() {
    assert_eq!(
        DomainEventKind::DeviceSessionOpened.event_type(),
        "iot.device.session.started"
    );
    assert_eq!(
        DomainEventKind::TelemetryRecorded.event_type(),
        "iot.telemetry.received"
    );
    assert_eq!(
        DomainEventKind::CommandDispatchRequested.event_type(),
        "iot.command.dispatchRequested"
    );
    assert_eq!(
        DomainEventKind::OtaCheckRequested.event_type(),
        "iot.firmware.otaCheckRequested"
    );
}
