use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::time::Duration;

fn main() {
    let shared_repository = std::sync::Arc::new(build_device_repository());
    let server = sdkwork_aiot_http_api::standard_admin_api_server()
        .expect("admin api server")
        .with_device_repository(shared_repository.clone())
        .with_command_repository(shared_repository.clone())
        .with_event_repository(shared_repository.clone())
        .with_twin_repository(shared_repository);
    let plan = sdkwork_aiot_runtime::RuntimeServicePlan::standard();

    println!(
        "sdkwork-aiot-admin-api mode={:?} backend_routes={} components={}",
        server.runtime().mode(),
        plan.backend_routes.len(),
        server.runtime().component_names().len()
    );

    if std::env::var("SDKWORK_AIOT_ADMIN_API_NO_LISTEN").as_deref() == Ok("1") {
        return;
    }

    let bind_addr = std::env::var("SDKWORK_AIOT_ADMIN_API_BIND")
        .unwrap_or_else(|_| "127.0.0.1:18081".to_string());
    serve(&server, &bind_addr);
}

fn build_device_repository() -> sdkwork_aiot_storage_sqlx::SqliteSqlxDeviceRepository {
    if let Some(path) = configured_device_db_path("SDKWORK_AIOT_ADMIN_API_DEVICE_DB_PATH") {
        ensure_parent_directory_exists(&path);
        println!("sdkwork-aiot-admin-api device-db=sqlite file={path}");
        return sdkwork_aiot_storage_sqlx::SqliteSqlxDeviceRepository::open(path)
            .expect("open sqlite aiot device repository");
    }

    println!("sdkwork-aiot-admin-api device-db=sqlite mode=memory");
    sdkwork_aiot_storage_sqlx::SqliteSqlxDeviceRepository::new_in_memory()
        .expect("sqlite aiot device repository")
}

fn configured_device_db_path(service_env_key: &str) -> Option<String> {
    std::env::var(service_env_key)
        .ok()
        .or_else(|| std::env::var("SDKWORK_AIOT_DEVICE_DB_PATH").ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn ensure_parent_directory_exists(path: &str) {
    let parent = Path::new(path).parent();
    if let Some(parent) = parent {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).expect("create sqlite parent directory");
        }
    }
}

fn serve(server: &sdkwork_aiot_http_api::AiotApiServer, bind_addr: &str) {
    let listener = TcpListener::bind(bind_addr).expect("bind admin api listener");
    println!("sdkwork-aiot-admin-api listening on http://{bind_addr}");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("sdkwork-aiot-admin-api accept_error={error}");
                continue;
            }
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

        let mut buffer = [0u8; 8192];
        let read = match stream.read(&mut buffer) {
            Ok(read) => read,
            Err(error) => {
                eprintln!("sdkwork-aiot-admin-api read_error={error}");
                continue;
            }
        };
        if read == 0 {
            continue;
        }

        let response =
            match sdkwork_aiot_http_api::handle_api_request_bytes(server, &buffer[..read]) {
                Ok(response) => response,
                Err(error) => problem_response(&error.code),
            };

        if let Err(error) = stream.write_all(response.as_bytes()) {
            eprintln!("sdkwork-aiot-admin-api write_error={error}");
        }
    }
}

fn problem_response(code: &str) -> String {
    let body = format!(
        r#"{{"type":"about:blank","title":"Bad Request","status":400,"code":"{}"}}"#,
        code
    );
    format!(
        "HTTP/1.1 400 Bad Request\r\ncontent-type: application/problem+json\r\ncontent-length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}
