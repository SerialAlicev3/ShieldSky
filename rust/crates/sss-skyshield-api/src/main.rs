use sss_skyshield_api::{build_router, AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let state_path = std::env::var("SSS_SKYSHIELD_STATE_PATH")
        .unwrap_or_else(|_| "data/sss-skyshield-state.json".to_string());
    let state = AppState::file_backed(state_path).expect("skyshield state should initialize");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4081")
        .await
        .expect("bind skyshield api");
    tracing::info!("sss-skyshield-api listening on 127.0.0.1:4081");
    axum::serve(listener, build_router(state))
        .await
        .expect("serve");
}
