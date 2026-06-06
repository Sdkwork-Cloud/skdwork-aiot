use std::fs;
use std::path::{Path, PathBuf};

use sdkwork_aiot_http_api::{standard_api_route_contracts, AiotApiSurface};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn quoted_json_values_after_key(document: &str, key: &str) -> Vec<String> {
    let marker = format!(r#""{key}": ""#);
    document
        .match_indices(&marker)
        .filter_map(|(start, _)| {
            let value_start = start + marker.len();
            let rest = &document[value_start..];
            let value_end = rest.find('"')?;
            Some(rest[..value_end].to_string())
        })
        .collect()
}

fn openapi_permission_for_operation(document: &str, operation_id: &str) -> Option<String> {
    let operation_marker = format!(r#""operationId": "{operation_id}""#);
    let operation_start = document.find(&operation_marker)?;
    let rest = &document[operation_start..];
    let permission_marker = r#""x-sdkwork-required-permission": ""#;
    let permission_start = rest.find(permission_marker)? + permission_marker.len();
    let permission_rest = &rest[permission_start..];
    let permission_end = permission_rest.find('"')?;

    Some(permission_rest[..permission_end].to_string())
}

#[test]
fn workspace_does_not_create_parallel_aiot_iam_component() {
    let root = workspace_root();
    let cargo = fs::read_to_string(root.join("Cargo.toml")).expect("workspace Cargo.toml");

    assert!(!cargo.contains("sdkwork-aiot-iam"));
    assert!(!root.join("crates").join("sdkwork-aiot-iam").exists());
}

#[test]
fn service_shells_reuse_runtime_builder_instead_of_owning_domain_logic() {
    let root = workspace_root();

    for service in [
        "services/sdkwork-aiot-gateway/src/main.rs",
        "services/sdkwork-aiot-admin-api/src/main.rs",
        "services/sdkwork-aiot-app-api/src/main.rs",
    ] {
        let source = fs::read_to_string(root.join(service)).expect(service);

        assert!(
            source.contains("standard_aiot_runtime")
                || source.contains("standard_standalone")
                || source.contains("standard_gateway_server")
                || source.contains("standard_admin_api_server")
                || source.contains("standard_app_api_server"),
            "{service} must assemble a shared runtime-backed component"
        );
        assert!(
            !source.contains("struct Device") && !source.contains("struct Product"),
            "{service} must not define domain entities"
        );
        assert!(
            !source.contains("CREATE TABLE"),
            "{service} must not own database DDL"
        );

        if service.contains("admin-api") || service.contains("app-api") {
            assert!(
                source.contains("sdkwork_aiot_http_api"),
                "{service} must route through the shared HTTP API component"
            );
            assert!(
                !source.contains("/backend/v3/api/iot/protocol_adapters")
                    && !source.contains("/app/v3/api/iot/devices"),
                "{service} must not inline app/backend API route behavior"
            );
        }
    }
}

#[test]
fn local_component_specs_exist_for_sdkwork_discovery() {
    let root = workspace_root();
    let readme = root.join("specs").join("README.md");
    let manifest = root.join("specs").join("component.spec.json");
    let manifest_text = fs::read_to_string(&manifest).expect("component spec manifest");

    assert!(readme.exists(), "specs/README.md is required");
    assert!(manifest.exists(), "specs/component.spec.json is required");
    assert!(manifest_text.contains(r#""kind": "sdkwork.component.spec""#));
    assert!(manifest_text.contains(r#""domain": "iot""#));
    assert!(manifest_text.contains(r#""type": "rust-crate""#));
    assert!(manifest_text.contains(r#""protocolPluginStandard""#));
    assert!(manifest_text.contains(r#""sdkwork_aiot_protocol::ProtocolAdapterManifest""#));
    assert!(manifest_text.contains(r#""codecs""#));
    assert!(manifest_text.contains(r#""session_policies""#));
    assert!(manifest_text.contains(r#""hardware_families""#));
    assert!(manifest_text.contains("API_SPEC.md"));
    assert!(manifest_text.contains("DATABASE_SPEC.md"));
    assert!(manifest_text.contains("COMPONENT_SPEC.md"));
}

#[test]
fn external_mqtt_broker_reference_is_rmqtt_only() {
    let root = workspace_root();
    let gitmodules = fs::read_to_string(root.join(".gitmodules")).expect(".gitmodules");

    assert!(
        gitmodules.contains(r#"[submodule "external/rmqtt"]"#),
        "rmqtt must be the canonical MQTT broker/server external implementation"
    );
    assert!(gitmodules.contains("https://github.com/rmqtt/rmqtt.git"));

    for removed in ["external/emqx", "external/mosquitto", "external/vernemq"] {
        assert!(
            !gitmodules.contains(removed),
            "{removed} must not remain as a MQTT broker external implementation"
        );
    }
}

#[test]
fn external_submodules_are_curated_high_signal_iot_references() {
    let root = workspace_root();
    let gitmodules = fs::read_to_string(root.join(".gitmodules")).expect(".gitmodules");

    let mut paths = gitmodules
        .lines()
        .filter_map(|line| line.trim().strip_prefix("path = "))
        .collect::<Vec<_>>();
    paths.sort_unstable();

    let mut expected = vec![
        "external/arduino-esp32",
        "external/esp-idf",
        "external/esphome",
        "external/micropython",
        "external/rmqtt",
        "external/tasmota",
        "external/thingsboard",
        "external/wled",
        "external/xiaozhi-esp32",
        "external/zephyr",
        "external/zigbee2mqtt",
    ];
    expected.sort_unstable();

    assert_eq!(
        paths, expected,
        "external submodules must stay focused on high-star smart-hardware references plus the explicit rmqtt MQTT implementation"
    );
}

#[test]
fn sdk_families_have_openapi_sources_and_generation_manifests() {
    let root = workspace_root();

    for (family, prefix, package_name) in [
        (
            "sdks/sdkwork-aiot-app-sdk",
            "/app/v3/api/iot",
            "@sdkwork/aiot-app-sdk",
        ),
        (
            "sdks/sdkwork-aiot-backend-sdk",
            "/backend/v3/api/iot",
            "@sdkwork/aiot-backend-sdk",
        ),
    ] {
        let family_root = root.join(family);
        let openapi = family_root.join("openapi").join(format!(
            "{}.openapi.json",
            family_root.file_name().unwrap().to_string_lossy()
        ));
        let sdkgen = family_root.join("openapi").join(format!(
            "{}.sdkgen.json",
            family_root.file_name().unwrap().to_string_lossy()
        ));
        let assembly = family_root.join(".sdkwork-assembly.json");

        let openapi_text = fs::read_to_string(&openapi).expect("openapi source");
        let sdkgen_text = fs::read_to_string(&sdkgen).expect("sdkgen manifest");
        let assembly_text = fs::read_to_string(&assembly).expect("sdk assembly");

        assert!(openapi_text.contains(r#""openapi": "3.1.2""#));
        assert!(openapi_text.contains(prefix));
        assert!(openapi_text.contains(r#""Authorization""#));
        assert!(openapi_text.contains(r#""Access-Token""#));
        assert!(openapi_text.contains(r#""X-Sdkwork-Tenant-Id""#));
        assert!(openapi_text.contains(r#""X-Sdkwork-Organization-Id""#));
        assert!(openapi_text.contains(r#""X-Sdkwork-User-Id""#));
        assert!(openapi_text.contains(r#""X-Sdkwork-Data-Scope""#));
        assert!(openapi_text.contains(r#""X-Sdkwork-Permission-Scope""#));
        assert!(openapi_text.contains(r#""x-sdkwork-required-permission""#));
        assert!(openapi_text.contains("application/problem+json"));
        assert!(
            openapi_text.contains(r#""operationId": "devices.list""#)
                || openapi_text.contains(r#""operationId": "products.list""#),
            "{family} must expose resource-style dotted operationIds"
        );
        if family.ends_with("backend-sdk") {
            for expected in [
                r#""operationId": "protocolAdapters.list""#,
                r#""operationId": "runtime.capacity.retrieve""#,
                r#""x-sdkwork-required-permission": "iot.protocolAdapters.read""#,
                r#""x-sdkwork-required-permission": "iot.runtime.read""#,
                r#""AiotProtocolAdapter""#,
                r#""AiotRuntimeCapacityPolicy""#,
                r#""securityModes""#,
                r#""sessionPolicies""#,
                r#""hardwareFamilies""#,
                r#""backpressure""#,
            ] {
                assert!(
                    openapi_text.contains(expected),
                    "{family} OpenAPI missing {expected}"
                );
            }
        }
        assert!(sdkgen_text.contains(r#""standardProfile": "sdkwork-v3""#));
        assert!(sdkgen_text.contains(package_name));
        assert!(sdkgen_text.contains(prefix));
        assert!(assembly_text.contains(package_name));
        assert!(assembly_text.contains(r#""generatedProtocols": ["http"]"#));
    }
}

#[test]
fn typescript_sdk_boundaries_are_reserved_for_generated_clients() {
    let root = workspace_root();

    for (package_path, package_name, client_name) in [
        (
            "sdks/sdkwork-aiot-app-sdk/sdkwork-aiot-app-sdk-typescript",
            "@sdkwork/aiot-app-sdk",
            "SdkworkAiotAppClient",
        ),
        (
            "sdks/sdkwork-aiot-backend-sdk/sdkwork-aiot-backend-sdk-typescript",
            "@sdkwork/aiot-backend-sdk",
            "SdkworkAiotBackendClient",
        ),
    ] {
        let package_root = root.join(package_path);
        let package_json = fs::read_to_string(package_root.join("package.json"))
            .expect("typescript sdk package.json");
        let sdk_json = fs::read_to_string(package_root.join("sdkwork-sdk.json"))
            .expect("typescript sdkwork-sdk.json");
        let index = fs::read_to_string(package_root.join("src").join("index.ts"))
            .expect("typescript sdk index");

        assert!(package_json.contains(package_name));
        assert!(sdk_json.contains(package_name));
        assert!(sdk_json.contains(r#""generated": true"#));
        assert!(index.contains(client_name));
        assert!(index.contains("Generated SDK placeholder"));
        assert!(
            !index.contains("fetch(") && !index.contains("XMLHttpRequest"),
            "reserved SDK boundary must not introduce handwritten transport logic"
        );
    }
}

#[test]
fn http_api_route_contracts_are_reflected_in_openapi_sources() {
    let root = workspace_root();

    for route in standard_api_route_contracts() {
        let openapi_path = match route.surface {
            AiotApiSurface::App => {
                "sdks/sdkwork-aiot-app-sdk/openapi/sdkwork-aiot-app-sdk.openapi.json"
            }
            AiotApiSurface::Admin => {
                "sdks/sdkwork-aiot-backend-sdk/openapi/sdkwork-aiot-backend-sdk.openapi.json"
            }
        };
        let openapi = fs::read_to_string(root.join(openapi_path)).expect(openapi_path);

        assert!(
            openapi.contains(&format!(r#""{}""#, route.path)),
            "{openapi_path} missing path {}",
            route.path
        );
        assert!(
            openapi.contains(&format!(r#""operationId": "{}""#, route.operation_id)),
            "{openapi_path} missing operationId {}",
            route.operation_id
        );
        assert!(
            openapi.contains(&format!(
                r#""x-sdkwork-required-permission": "{}""#,
                route.required_permission
            )),
            "{openapi_path} missing required permission {} for {}",
            route.required_permission,
            route.operation_id
        );
    }
}

#[test]
fn openapi_operations_are_reflected_in_http_api_route_contracts() {
    let root = workspace_root();
    let contracts = standard_api_route_contracts();

    for (surface, openapi_path) in [
        (
            AiotApiSurface::App,
            "sdks/sdkwork-aiot-app-sdk/openapi/sdkwork-aiot-app-sdk.openapi.json",
        ),
        (
            AiotApiSurface::Admin,
            "sdks/sdkwork-aiot-backend-sdk/openapi/sdkwork-aiot-backend-sdk.openapi.json",
        ),
    ] {
        let openapi = fs::read_to_string(root.join(openapi_path)).expect(openapi_path);

        for operation_id in quoted_json_values_after_key(&openapi, "operationId") {
            assert!(
                contracts.iter().any(|route| {
                    route.surface == surface && route.operation_id == operation_id
                }),
                "{openapi_path} operationId {operation_id} missing from Rust route contracts"
            );
        }
    }
}

#[test]
fn openapi_operation_permissions_match_http_api_route_contracts() {
    let root = workspace_root();
    let contracts = standard_api_route_contracts();

    for route in contracts {
        let openapi_path = match route.surface {
            AiotApiSurface::App => {
                "sdks/sdkwork-aiot-app-sdk/openapi/sdkwork-aiot-app-sdk.openapi.json"
            }
            AiotApiSurface::Admin => {
                "sdks/sdkwork-aiot-backend-sdk/openapi/sdkwork-aiot-backend-sdk.openapi.json"
            }
        };
        let openapi = fs::read_to_string(root.join(openapi_path)).expect(openapi_path);
        let permission = openapi_permission_for_operation(&openapi, route.operation_id)
            .unwrap_or_else(|| {
                panic!(
                    "{openapi_path} missing permission for {}",
                    route.operation_id
                )
            });

        assert_eq!(
            permission, route.required_permission,
            "{openapi_path} permission mismatch for {}",
            route.operation_id
        );
    }
}

#[test]
fn backend_openapi_uses_media_resource_contract_for_firmware_artifact_io() {
    let root = workspace_root();
    let backend_openapi = fs::read_to_string(
        root.join("sdks/sdkwork-aiot-backend-sdk/openapi/sdkwork-aiot-backend-sdk.openapi.json"),
    )
    .expect("backend openapi");

    assert!(backend_openapi.contains(r#""AiotFirmwareArtifactCreateRequest""#));
    assert!(backend_openapi.contains(r#""resource": {"#));
    assert!(backend_openapi.contains(r##""$ref": "#/components/schemas/MediaResource""##));
    assert!(backend_openapi.contains(r#""MediaKind""#));
    assert!(backend_openapi.contains(r#""MediaSource""#));
    assert!(backend_openapi.contains(r#""MediaAccess""#));
    assert!(backend_openapi.contains(r#""MediaChecksum""#));
    assert!(
        !backend_openapi.contains(r#""storageUri""#),
        "firmware artifact MediaResource contract must not expose bare storageUri fields"
    );
}

#[test]
fn event_openapi_contracts_use_typed_event_payload_and_media_resource_fields() {
    let root = workspace_root();
    let app_openapi = fs::read_to_string(
        root.join("sdks/sdkwork-aiot-app-sdk/openapi/sdkwork-aiot-app-sdk.openapi.json"),
    )
    .expect("app openapi");
    let backend_openapi = fs::read_to_string(
        root.join("sdks/sdkwork-aiot-backend-sdk/openapi/sdkwork-aiot-backend-sdk.openapi.json"),
    )
    .expect("backend openapi");

    assert!(app_openapi.contains(r#""AiotEventListResponse""#));
    assert!(app_openapi.contains(r#""AiotEvent""#));
    assert!(app_openapi.contains(r##""$ref": "#/components/schemas/AiotEvent""##));
    assert!(app_openapi.contains(r##""$ref": "#/components/schemas/MediaResource""##));
    assert!(!app_openapi.contains(r#""eventImageUrl""#));
    assert!(!app_openapi.contains(r#""eventAudioUrl""#));

    assert!(backend_openapi.contains(r#""AiotEventListResponse""#));
    assert!(backend_openapi.contains(r#""AiotEvent""#));
    assert!(backend_openapi.contains(r##""$ref": "#/components/schemas/AiotEventListResponse""##));
    assert!(backend_openapi.contains(r##""$ref": "#/components/schemas/MediaResource""##));
    assert!(!backend_openapi.contains(r#""eventImageUrl""#));
    assert!(!backend_openapi.contains(r#""eventAudioUrl""#));
}

#[test]
fn command_openapi_contracts_use_media_resource_for_request_and_result_payloads() {
    let root = workspace_root();
    let app_openapi = fs::read_to_string(
        root.join("sdks/sdkwork-aiot-app-sdk/openapi/sdkwork-aiot-app-sdk.openapi.json"),
    )
    .expect("app openapi");
    let backend_openapi = fs::read_to_string(
        root.join("sdks/sdkwork-aiot-backend-sdk/openapi/sdkwork-aiot-backend-sdk.openapi.json"),
    )
    .expect("backend openapi");

    assert!(app_openapi.contains(r#""AiotCommandCreateRequest""#));
    assert!(app_openapi.contains(r#""AiotCommandResponse""#));
    assert!(app_openapi.contains(r#""AiotCommandResult""#));
    assert!(app_openapi.contains(r#""requestMediaResourceId""#));
    assert!(app_openapi.contains(r#""resultMediaResourceId""#));
    assert!(app_openapi.contains(r##""$ref": "#/components/schemas/MediaResource""##));
    assert!(!app_openapi.contains(r#""requestAudioUrl""#));
    assert!(!app_openapi.contains(r#""resultAudioUrl""#));

    assert!(backend_openapi.contains(r#""AiotCommandListResponse""#));
    assert!(backend_openapi.contains(r#""AiotCommand""#));
    assert!(backend_openapi.contains(r#""AiotCommandResult""#));
    assert!(backend_openapi.contains(r#""requestMediaResourceId""#));
    assert!(backend_openapi.contains(r#""resultMediaResourceId""#));
    assert!(backend_openapi.contains(r##""$ref": "#/components/schemas/MediaResource""##));
    assert!(!backend_openapi.contains(r#""requestAudioUrl""#));
    assert!(!backend_openapi.contains(r#""resultAudioUrl""#));
}

#[test]
fn declared_http_collection_routes_are_mounted_by_shared_api_component() {
    let http_api =
        fs::read_to_string(workspace_root().join("crates/sdkwork-aiot-http-api/src/lib.rs"))
            .expect("http api source");

    for route in standard_api_route_contracts() {
        if route.method == "GET"
            && !route.path.contains('{')
            && route.operation_id.ends_with(".list")
        {
            assert!(
                http_api.contains(route.path),
                "HTTP API component must mount declared collection route {}",
                route.path
            );
        }
    }
}

#[test]
fn crate_dependency_boundaries_do_not_invert_architecture() {
    let root = workspace_root();

    for crate_manifest in [
        "crates/sdkwork-aiot-contract/Cargo.toml",
        "crates/sdkwork-aiot-core/Cargo.toml",
        "crates/sdkwork-aiot-protocol/Cargo.toml",
        "crates/sdkwork-aiot-runtime/Cargo.toml",
        "crates/sdkwork-aiot-storage/Cargo.toml",
        "crates/sdkwork-aiot-storage-sqlx/Cargo.toml",
        "crates/sdkwork-aiot-security/Cargo.toml",
        "crates/sdkwork-aiot-observability/Cargo.toml",
        "crates/sdkwork-aiot-adapter-xiaozhi/Cargo.toml",
        "crates/sdkwork-aiot-transport/Cargo.toml",
        "crates/sdkwork-aiot-http-api/Cargo.toml",
    ] {
        let manifest = fs::read_to_string(root.join(crate_manifest)).expect(crate_manifest);

        assert!(
            !manifest.contains("services/"),
            "{crate_manifest} must not depend on service binaries"
        );
        assert!(
            !manifest.contains("sdkwork-appbase"),
            "{crate_manifest} must not depend on appbase concrete IAM packages"
        );
    }

    let adapter_manifest =
        fs::read_to_string(root.join("crates/sdkwork-aiot-adapter-xiaozhi/Cargo.toml"))
            .expect("xiaozhi manifest");
    assert!(
        !adapter_manifest.contains("sdkwork-aiot-storage-sqlx")
            && !adapter_manifest.contains("sqlx"),
        "protocol adapters must not depend on storage implementations"
    );

    let transport_manifest =
        fs::read_to_string(root.join("crates/sdkwork-aiot-transport/Cargo.toml"))
            .expect("transport manifest");
    assert!(
        !transport_manifest.contains("sdkwork-aiot-adapter-xiaozhi"),
        "transport must stay protocol-neutral and accept codec/plugin injection"
    );
}

#[test]
fn protocol_plugin_manifest_standard_fields_are_not_eroded() {
    let root = workspace_root();
    let protocol_source = fs::read_to_string(root.join("crates/sdkwork-aiot-protocol/src/lib.rs"))
        .expect("protocol source");
    let xiaozhi_source =
        fs::read_to_string(root.join("crates/sdkwork-aiot-adapter-xiaozhi/src/lib.rs"))
            .expect("xiaozhi source");

    for expected in [
        "pub enum CodecKind",
        "pub enum SessionPolicy",
        "pub scope: ProtocolPluginScope",
        "pub codecs: Vec<CodecKind>",
        "pub session_policies: Vec<SessionPolicy>",
        "pub hardware_families: Vec<String>",
        "pub runtime_profiles: Vec<String>",
        "pub firmware_profiles: Vec<String>",
        "pub fn with_scope",
        "pub fn with_codec",
        "pub fn with_session_policy",
        "pub fn with_hardware_family",
    ] {
        assert!(
            protocol_source.contains(expected),
            "protocol manifest standard missing {expected}"
        );
    }

    for expected in [
        "with_scope(ProtocolPluginScope::CompatibilityPlugin)",
        "with_codec(CodecKind::JsonText)",
        "with_codec(CodecKind::JsonRpc)",
        "with_codec(CodecKind::BinaryMedia)",
        "with_session_policy(SessionPolicy::StatefulDeviceSession)",
        "with_hardware_family(\"esp32\")",
        "with_runtime_profile(\"esp_idf\")",
        "with_firmware_profile(\"xiaozhi_ota\")",
    ] {
        assert!(
            xiaozhi_source.contains(expected),
            "xiaozhi plugin manifest missing {expected}"
        );
    }
}
