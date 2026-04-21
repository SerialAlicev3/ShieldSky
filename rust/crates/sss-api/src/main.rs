use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use sss_api::{build_router, state::PassiveRegionRunRequest, AppState, WebhookDeliveryPolicy};
use sss_storage::SqliteStore;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sss_api=info,tower_http=info".into()),
        )
        .init();

    let bind = if let Ok(port) = env::var("PORT") {
        format!("0.0.0.0:{port}")
    } else {
        env::var("SSS_API_BIND").unwrap_or_else(|_| "127.0.0.1:8088".to_string())
    };
    let addr: SocketAddr = bind.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    let storage_path = storage_path();
    ensure_storage_parent(&storage_path)?;
    let storage = SqliteStore::open(&storage_path)?;
    tracing::info!("sss-api listening on http://{local_addr}");
    tracing::info!("sss-api storage at {}", storage_path.display());
    let state = configure_state_from_env(AppState::from_storage(storage)?);
    if let Some(config) = scheduler_config() {
        tracing::info!(
            poll_seconds = config.poll_interval.as_secs(),
            retry_seconds = config.retry_interval.as_secs(),
            "starting CelesTrak active ingest scheduler"
        );
        tokio::spawn(run_celestrak_scheduler(state.clone(), config));
    }
    if let Some(config) = passive_region_scheduler_config() {
        tracing::info!(
            poll_seconds = config.poll_interval.as_secs(),
            retry_seconds = config.retry_interval.as_secs(),
            window_hours = config.window_hours,
            include_adsb = config.feeds.include_adsb,
            include_weather = config.feeds.include_weather,
            include_fire_smoke = config.feeds.include_fire_smoke,
            force_discovery = config.force_discovery,
            dry_run = config.dry_run,
            "starting passive region scheduler"
        );
        tokio::spawn(run_passive_region_scheduler(state.clone(), config));
    }

    axum::serve(listener, build_router(state)).await?;
    Ok(())
}

fn configure_state_from_env(mut state: AppState) -> AppState {
    if let Some(api_key) = non_empty_env("SSS_NASA_API_KEY") {
        tracing::info!("sss-api nasa api key configured");
        state = state.with_nasa_api_key(api_key);
    }
    if let Some(base_url) = non_empty_env("SSS_NASA_API_BASE_URL") {
        tracing::info!(%base_url, "sss-api nasa api base url configured");
        state = state.with_nasa_api_base_url(base_url);
    }
    if let Some(base_url) = non_empty_env("SSS_OPEN_METEO_BASE_URL") {
        tracing::info!(%base_url, "sss-api open-meteo base url configured");
        state = state.with_open_meteo_base_url(base_url);
    }
    if let Some(base_url) = non_empty_env("SSS_FIRMS_API_BASE_URL") {
        tracing::info!(%base_url, "sss-api firms api base url configured");
        state = state.with_firms_api_base_url(base_url);
    }
    if let Some(map_key) = non_empty_env("SSS_FIRMS_MAP_KEY") {
        tracing::info!("sss-api firms map key configured");
        state = state.with_firms_map_key(map_key);
    }
    if let Some(source) = non_empty_env("SSS_FIRMS_SOURCE") {
        tracing::info!(%source, "sss-api firms source configured");
        state = state.with_firms_source(source);
    }
    if let Some(base_url) = non_empty_env("SSS_OPENSKY_API_BASE_URL") {
        tracing::info!(%base_url, "sss-api opensky api base url configured");
        state = state.with_opensky_api_base_url(base_url);
    }
    if let Some(token) = non_empty_env("SSS_OPENSKY_BEARER_TOKEN") {
        tracing::info!("sss-api opensky bearer token configured");
        state = state.with_opensky_bearer_token(token);
    }
    if let Some(base_url) = non_empty_env("SSS_OVERPASS_API_BASE_URL") {
        tracing::info!(%base_url, "sss-api overpass api base url configured");
        state = state.with_overpass_api_base_url(base_url);
    }
    if let Some(url) = non_empty_env("SSS_WEBHOOK_ENDPOINT") {
        tracing::info!(%url, "sss-api webhook delivery enabled");
        state = state.with_webhook_endpoint(url);
        let policy = webhook_delivery_policy();
        tracing::info!(
            max_attempts = policy.max_attempts,
            retry_delay_ms = policy.retry_delay.as_millis(),
            "sss-api webhook retry policy configured"
        );
        state = state.with_webhook_delivery_policy(policy);
    }
    state
}

fn non_empty_env(key: &str) -> Option<String> {
    env::var(key).ok().filter(|value| !value.trim().is_empty())
}

fn storage_path() -> PathBuf {
    env::var_os("SSS_STORAGE_PATH")
        .map_or_else(|| PathBuf::from("data/sss-api.sqlite"), PathBuf::from)
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

#[derive(Debug, Clone, Copy)]
struct SchedulerConfig {
    poll_interval: Duration,
    retry_interval: Duration,
}

#[derive(Debug, Clone, Copy)]
struct PassiveRegionSchedulerConfig {
    poll_interval: Duration,
    retry_interval: Duration,
    window_hours: u64,
    feeds: PassiveRegionFeedConfig,
    force_discovery: bool,
    dry_run: bool,
}

#[derive(Debug, Clone, Copy)]
struct PassiveRegionFeedConfig {
    include_adsb: bool,
    include_weather: bool,
    include_fire_smoke: bool,
}

fn scheduler_config() -> Option<SchedulerConfig> {
    let poll_seconds = env::var("SSS_CELESTRAK_ACTIVE_POLL_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    if poll_seconds == 0 {
        return None;
    }

    let retry_seconds = env::var("SSS_CELESTRAK_ACTIVE_RETRY_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(60);

    Some(SchedulerConfig {
        poll_interval: Duration::from_secs(poll_seconds),
        retry_interval: Duration::from_secs(retry_seconds.max(1)),
    })
}

fn passive_region_scheduler_config() -> Option<PassiveRegionSchedulerConfig> {
    let poll_seconds = env::var("SSS_PASSIVE_REGION_POLL_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    if poll_seconds == 0 {
        return None;
    }

    let retry_seconds = env::var("SSS_PASSIVE_REGION_RETRY_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(300);
    let window_hours = env::var("SSS_PASSIVE_REGION_WINDOW_HOURS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(24)
        .clamp(1, 168);

    Some(PassiveRegionSchedulerConfig {
        poll_interval: Duration::from_secs(poll_seconds.max(1)),
        retry_interval: Duration::from_secs(retry_seconds.max(1)),
        window_hours,
        feeds: PassiveRegionFeedConfig {
            include_adsb: bool_env("SSS_PASSIVE_REGION_INCLUDE_ADSB", true),
            include_weather: bool_env("SSS_PASSIVE_REGION_INCLUDE_WEATHER", true),
            include_fire_smoke: bool_env("SSS_PASSIVE_REGION_INCLUDE_FIRE_SMOKE", true),
        },
        force_discovery: bool_env("SSS_PASSIVE_REGION_FORCE_DISCOVERY", false),
        dry_run: bool_env("SSS_PASSIVE_REGION_DRY_RUN", false),
    })
}

fn bool_env(key: &str, default: bool) -> bool {
    env::var(key).ok().map_or(default, |value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

fn webhook_delivery_policy() -> WebhookDeliveryPolicy {
    let max_attempts = env::var("SSS_WEBHOOK_RETRY_ATTEMPTS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(3)
        .max(1);
    let retry_delay_ms = env::var("SSS_WEBHOOK_RETRY_DELAY_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(500)
        .max(1);

    WebhookDeliveryPolicy {
        max_attempts,
        retry_delay: Duration::from_millis(retry_delay_ms),
    }
}

async fn run_celestrak_scheduler(state: AppState, config: SchedulerConfig) {
    let source = "celestrak-active";
    let url = sss_ingest::celestrak_active_url();
    let mut delay = Duration::from_secs(0);

    loop {
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }

        let request_id = format!("scheduler-{source}-{}", unix_seconds_now());
        match state.ingest_live_tle_source(&request_id, source, url).await {
            Ok((payload, outcome)) => {
                tracing::info!(
                    %request_id,
                    %source,
                    payload_bytes = payload.len(),
                    records_received = outcome.records_received,
                    observations_created = outcome.observations_created,
                    skipped_duplicate = outcome.skipped_duplicate,
                    freshness_seconds = outcome.freshness_seconds,
                    "scheduled ingest completed"
                );
                delay = config.poll_interval;
            }
            Err(error) => {
                tracing::warn!(
                    %request_id,
                    %source,
                    retry_seconds = config.retry_interval.as_secs(),
                    error = %error,
                    "scheduled ingest failed"
                );
                delay = config.retry_interval;
            }
        }
    }
}

async fn run_passive_region_scheduler(state: AppState, config: PassiveRegionSchedulerConfig) {
    let mut delay = Duration::from_secs(0);

    loop {
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }

        let request_id = format!("scheduler-passive-regions-{}", unix_seconds_now());
        let request = PassiveRegionRunRequest {
            region_ids: None,
            force_discovery: Some(config.force_discovery),
            dry_run: Some(config.dry_run),
            window_hours: Some(config.window_hours),
            include_adsb: Some(config.feeds.include_adsb),
            include_weather: Some(config.feeds.include_weather),
            include_fire_smoke: Some(config.feeds.include_fire_smoke),
        };

        match state.run_passive_regions(&request_id, &request).await {
            Ok(response) => {
                tracing::info!(
                    %request_id,
                    evaluated_regions = response.evaluated_region_count,
                    discovered_regions = response.discovered_region_count,
                    skipped_regions = response.skipped_region_count,
                    discovered_seeds = response.discovered_seed_count,
                    scheduler_selected_seeds = response
                        .scheduler
                        .as_ref()
                        .map_or(0, |scheduler| scheduler.selected_seed_count),
                    "scheduled passive region run completed"
                );
                delay = config.poll_interval;
            }
            Err(error) => {
                tracing::warn!(
                    %request_id,
                    retry_seconds = config.retry_interval.as_secs(),
                    error = %error,
                    "scheduled passive region run failed"
                );
                delay = config.retry_interval;
            }
        }
    }
}

fn unix_seconds_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .try_into()
        .unwrap_or(i64::MAX)
}
