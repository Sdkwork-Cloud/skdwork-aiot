# Repository Guidelines

## Project Structure & Module Organization

This repository is a Rust workspace for the SDKWork AIoT server. Shared libraries live under `crates/`, including contracts, protocol, runtime, storage, security, observability, transport, HTTP API, architecture checks, and the `sdkwork-aiot-adapter-xiaozhi` integration. Runnable services live under `services/`: `sdkwork-aiot-gateway`, `sdkwork-aiot-admin-api`, and `sdkwork-aiot-app-api`. Tests are colocated in each crate or service under `tests/`, usually with `*_standard.rs` names. Generated or packaged SDK artifacts live in `sdks/`; specification inputs live in `specs/`; design and planning notes live in `docs/`. The `external/` tree contains reference projects and submodules and should not be edited for normal product changes.

## Build, Test, and Development Commands

- `cargo build --workspace`: compile all workspace crates and services.
- `cargo test --workspace`: run the full Rust test suite.
- `cargo test -p sdkwork-aiot-gateway`: run tests for one package.
- `cargo run -p sdkwork-aiot-gateway`: start the local gateway service.
- `cargo run -p sdkwork-aiot-xiaozhi-simulator-ui`: launch the cross-platform Xiaozhi simulator UI.
- PowerShell gateway bind example: `$env:SDKWORK_AIOT_GATEWAY_BIND='127.0.0.1:18080'; cargo run -p sdkwork-aiot-gateway`.
- Optional persistent device DB: `$env:SDKWORK_AIOT_DEVICE_DB_PATH='D:\\data\\aiot-device.db'`.

## Coding Style & Naming Conventions

Use Rust 2021 idioms and keep modules small, typed, and explicit. Run `cargo fmt --all` before submitting changes. Prefer `snake_case` for modules, functions, variables, and test names; use `PascalCase` for structs, enums, and traits; use `SCREAMING_SNAKE_CASE` for constants. Package names follow the existing `sdkwork-aiot-*` pattern. Keep public APIs documented when they define cross-crate behavior.

## Testing Guidelines

Use Rust integration tests in each package's `tests/` directory. Name test files by behavior or standard surface, for example `xiaozhi_standard.rs`, `gateway_standard.rs`, or `transport_standard.rs`. Add focused tests for protocol compatibility, gateway routing, adapter parsing, and error cases before changing behavior. Run the narrow package test first, then `cargo test --workspace` before opening a pull request.

## Commit & Pull Request Guidelines

Recent commits use short imperative summaries such as `Model Raspberry Pi hardware gateway profiles` and `Implement SDKWork AIoT server foundation`. Follow that style: one clear sentence, no trailing period. Pull requests should include a concise description, affected crates or services, test evidence, and any configuration or protocol compatibility notes. Include screenshots only for browser-facing changes such as the Xiaozhi simulator.

## Security & Configuration Tips

Do not commit real device tokens, broker credentials, certificates, or local bind secrets. Prefer environment variables for service configuration. When changing Xiaozhi access paths, verify both real-device headers and browser simulator query-parameter compatibility.
