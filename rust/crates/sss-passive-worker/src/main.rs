use std::env;
use std::path::Path;

use sss_api::AppState;
use sss_passive_worker::{PassiveWorker, WorkerConfig};
use sss_storage::SqliteStore;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sss_passive_worker=info,sss_api=info".into()),
        )
        .init();

    let config = WorkerConfig::from_env();
    ensure_storage_parent(&config.storage_path)?;
    let storage = SqliteStore::open(&config.storage_path)?;
    let state = configure_state_from_env(AppState::from_storage(storage)?);
    let worker = PassiveWorker::new(state, config.clone());

    tracing::info!(
        storage_path = %config.storage_path.display(),
        poll_seconds = config.poll_interval_seconds,
        max_parallel_regions = config.max_parallel_regions,
        max_runtime_seconds = config.max_runtime_seconds,
        heartbeat_retention_seconds = config.heartbeat_retention_seconds,
        enable_discovery = config.enable_discovery,
        enable_scan = config.enable_scan,
        "sss passive worker starting"
    );

    tokio::select! {
        result = worker.run_forever() => result?,
        signal = tokio::signal::ctrl_c() => {
            signal?;
            tracing::info!("sss passive worker shutdown requested");
        }
    }

    Ok(())
}

fn configure_state_from_env(mut state: AppState) -> AppState {
    if let Some(api_key) = non_empty_env("SSS_NASA_API_KEY") {
        state = state.with_nasa_api_key(api_key);
    }
    if let Some(base_url) = non_empty_env("SSS_NASA_API_BASE_URL") {
        state = state.with_nasa_api_base_url(base_url);
    }
    if let Some(base_url) = non_empty_env("SSS_OPEN_METEO_BASE_URL") {
        state = state.with_open_meteo_base_url(base_url);
    }
    if let Some(base_url) = non_empty_env("SSS_FIRMS_API_BASE_URL") {
        state = state.with_firms_api_base_url(base_url);
    }
    if let Some(map_key) = non_empty_env("SSS_FIRMS_MAP_KEY") {
        state = state.with_firms_map_key(map_key);
    }
    if let Some(source) = non_empty_env("SSS_FIRMS_SOURCE") {
        state = state.with_firms_source(source);
    }
    if let Some(base_url) = non_empty_env("SSS_OPENSKY_API_BASE_URL") {
        state = state.with_opensky_api_base_url(base_url);
    }
    if let Some(token) = non_empty_env("SSS_OPENSKY_BEARER_TOKEN") {
        state = state.with_opensky_bearer_token(token);
    }
    if let Some(base_url) = non_empty_env("SSS_OVERPASS_API_BASE_URL") {
        state = state.with_overpass_api_base_url(base_url);
    }
    state
}

fn non_empty_env(key: &str) -> Option<String> {
    env::var(key).ok().filter(|value| !value.trim().is_empty())
}

fn ensure_storage_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
