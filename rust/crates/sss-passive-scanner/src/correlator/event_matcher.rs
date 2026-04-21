use serde_json::json;
use sss_site_registry::{Criticality, SiteType};
use uuid::Uuid;

use crate::domain::{
    PassiveEvent, PassiveObservation, PassiveSiteProfile, PassiveSourceKind, PassiveThreatType,
};

#[must_use]
pub fn match_events(
    sites: &[PassiveSiteProfile],
    observations: &[PassiveObservation],
) -> Vec<PassiveEvent> {
    let mut events = Vec::new();

    for site in sites {
        for observation in observations {
            let distance_to_site_km = haversine_km(
                site.site.latitude,
                site.site.longitude,
                observation.latitude,
                observation.longitude,
            );
            if distance_to_site_km > observation.coverage_radius_km + site.observation_radius_km {
                continue;
            }

            let threat_type = classify_threat(observation, site.site.site_type);
            let proximity_factor = 1.0
                - (distance_to_site_km
                    / (observation.coverage_radius_km + site.observation_radius_km))
                    .clamp(0.0, 1.0);
            let risk_score = (observation.severity_hint
                * criticality_weight(site.site.criticality)
                * site_type_weight(site.site.site_type)
                * (0.45 + (proximity_factor * 0.55)))
                .clamp(0.0, 1.0);
            let phenomenon_window_seconds = phenomenon_window_seconds(observation.source_kind);
            let phenomenon_time_bucket = observation
                .observed_at_unix_seconds
                .div_euclid(phenomenon_window_seconds);
            let phenomenon_anchor = source_identity_anchor(observation);

            events.push(PassiveEvent {
                event_id: Uuid::new_v4(),
                site_id: site.site.site_id,
                site_name: site.site.name.clone(),
                source_kind: observation.source_kind,
                threat_type,
                phenomenon_key: format!(
                    "{}:{:?}:{:?}:{}:{}",
                    site.site.site_id,
                    observation.source_kind,
                    threat_type,
                    phenomenon_anchor,
                    phenomenon_time_bucket
                ),
                observed_at_unix_seconds: observation.observed_at_unix_seconds,
                distance_to_site_km,
                risk_score,
                summary: summarize_event(observation, threat_type, &site.site.name),
                payload: event_payload(
                    observation,
                    &site.site.name,
                    distance_to_site_km,
                    &phenomenon_anchor,
                    phenomenon_window_seconds,
                    phenomenon_time_bucket,
                ),
            });
        }
    }

    events.sort_by(|left, right| {
        right
            .risk_score
            .partial_cmp(&left.risk_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    events
}

fn event_payload(
    observation: &PassiveObservation,
    site_name: &str,
    distance_to_site_km: f64,
    phenomenon_anchor: &str,
    phenomenon_window_seconds: i64,
    phenomenon_time_bucket: i64,
) -> serde_json::Value {
    let mut payload = observation.payload.clone();
    let object = payload
        .as_object_mut()
        .expect("passive observation payload should be a JSON object");
    object.insert(
        "observation_id".to_string(),
        json!(observation.observation_id),
    );
    object.insert("latitude".to_string(), json!(observation.latitude));
    object.insert("longitude".to_string(), json!(observation.longitude));
    object.insert("altitude_m".to_string(), json!(observation.altitude_m));
    object.insert(
        "distance_to_site_km".to_string(),
        json!(distance_to_site_km),
    );
    object.insert("site_name".to_string(), json!(site_name));
    object.insert("phenomenon_anchor".to_string(), json!(phenomenon_anchor));
    object.insert(
        "phenomenon_window_seconds".to_string(),
        json!(phenomenon_window_seconds),
    );
    object.insert(
        "phenomenon_time_bucket".to_string(),
        json!(phenomenon_time_bucket),
    );
    payload
}

fn phenomenon_window_seconds(source_kind: PassiveSourceKind) -> i64 {
    match source_kind {
        PassiveSourceKind::Adsb | PassiveSourceKind::Weather | PassiveSourceKind::FireSmoke => {
            6 * 3_600
        }
        PassiveSourceKind::Notam
        | PassiveSourceKind::Orbital
        | PassiveSourceKind::Satellite
        | PassiveSourceKind::InfraMap => 24 * 3_600,
    }
}

fn source_identity_anchor(observation: &PassiveObservation) -> String {
    match observation.source_kind {
        PassiveSourceKind::Adsb => payload_string(&observation.payload, "flight_id", "flight"),
        PassiveSourceKind::Notam => payload_string(&observation.payload, "notam_id", "notam"),
        PassiveSourceKind::Orbital => payload_string(&observation.payload, "object_id", "object"),
        PassiveSourceKind::Satellite | PassiveSourceKind::InfraMap => {
            payload_string(&observation.payload, "scene_id", "scene")
        }
        PassiveSourceKind::Weather | PassiveSourceKind::FireSmoke => Some(geo_anchor(
            observation,
            coarse_geo_bucket_degrees(observation.source_kind),
        )),
    }
    .unwrap_or_else(|| geo_anchor(observation, 0.25))
}

fn payload_string(payload: &serde_json::Value, key: &str, prefix: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(|value| format!("{prefix}:{value}"))
}

fn geo_anchor(observation: &PassiveObservation, bucket_degrees: f64) -> String {
    format!(
        "geo:{}:{}",
        coordinate_bucket(observation.latitude, bucket_degrees),
        coordinate_bucket(observation.longitude, bucket_degrees)
    )
}

fn coarse_geo_bucket_degrees(source_kind: PassiveSourceKind) -> f64 {
    match source_kind {
        PassiveSourceKind::Weather | PassiveSourceKind::FireSmoke | PassiveSourceKind::Orbital => {
            0.5
        }
        PassiveSourceKind::Adsb
        | PassiveSourceKind::Satellite
        | PassiveSourceKind::Notam
        | PassiveSourceKind::InfraMap => 0.25,
    }
}

#[allow(clippy::cast_possible_truncation)]
fn coordinate_bucket(value: f64, bucket_degrees: f64) -> i64 {
    (value / bucket_degrees).floor() as i64
}

fn classify_threat(observation: &PassiveObservation, site_type: SiteType) -> PassiveThreatType {
    match observation.source_kind {
        PassiveSourceKind::Adsb => PassiveThreatType::LowAltitudeOverflight,
        PassiveSourceKind::Notam => PassiveThreatType::RestrictedAirspaceActivity,
        PassiveSourceKind::Orbital => PassiveThreatType::OrbitalReentryContext,
        PassiveSourceKind::Satellite | PassiveSourceKind::InfraMap => {
            PassiveThreatType::SurfaceChangeDetected
        }
        PassiveSourceKind::FireSmoke => PassiveThreatType::SmokeExposure,
        PassiveSourceKind::Weather => {
            let hail_probability = observation
                .payload
                .get("hail_probability")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or_default();
            let wind_kph = observation
                .payload
                .get("wind_kph")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or_default();
            let irradiance_drop_ratio = observation
                .payload
                .get("irradiance_drop_ratio")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or_default();

            if hail_probability >= 0.5 {
                PassiveThreatType::HailExposure
            } else if matches!(site_type, SiteType::SolarPlant) && irradiance_drop_ratio >= 0.35 {
                PassiveThreatType::IrradianceDropExpected
            } else if wind_kph >= 70.0 {
                PassiveThreatType::WindLoadRisk
            } else {
                PassiveThreatType::IrradianceDropExpected
            }
        }
    }
}

fn summarize_event(
    observation: &PassiveObservation,
    threat_type: PassiveThreatType,
    site_name: &str,
) -> String {
    format!(
        "{:?} from {:?} observed near {}: {}",
        threat_type, observation.source_kind, site_name, observation.summary
    )
}

fn criticality_weight(criticality: Criticality) -> f64 {
    match criticality {
        Criticality::Low => 0.55,
        Criticality::Medium => 0.75,
        Criticality::High => 0.9,
        Criticality::Critical => 1.0,
    }
}

fn site_type_weight(site_type: SiteType) -> f64 {
    match site_type {
        SiteType::SolarPlant => 0.9,
        SiteType::DataCenter => 1.0,
        SiteType::Substation => 0.95,
    }
}

fn haversine_km(
    origin_latitude: f64,
    origin_longitude: f64,
    target_latitude: f64,
    target_longitude: f64,
) -> f64 {
    let delta_latitude = (target_latitude - origin_latitude).to_radians();
    let delta_longitude = (target_longitude - origin_longitude).to_radians();
    let origin_latitude_radians = origin_latitude.to_radians();
    let target_latitude_radians = target_latitude.to_radians();

    let sin_latitude = (delta_latitude / 2.0).sin();
    let sin_longitude = (delta_longitude / 2.0).sin();
    let a = sin_latitude * sin_latitude
        + origin_latitude_radians.cos()
            * target_latitude_radians.cos()
            * sin_longitude
            * sin_longitude;
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    6_371.0 * c
}
