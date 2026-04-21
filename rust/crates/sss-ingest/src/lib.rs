//! Real-data ingestion primitives for Serial Alice Sky.
//!
//! The MVP starts with public CelesTrak/NORAD-style TLE records and converts
//! them into SSS core objects plus normalized catalog observations.

use serde::{Deserialize, Serialize};
use sss_core::{
    BehaviorSignature, MissionClass, Observation, ObservationSource, OrbitRegime, OrbitalState,
    RiskProfile, SpaceObject, TleEphemeris, Vector3,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TleRecord {
    pub name: String,
    pub line1: String,
    pub line2: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedTleObservation {
    pub object: SpaceObject,
    pub observation: Observation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IngestError {
    MissingLine(&'static str),
    InvalidLine(&'static str),
    InvalidField(&'static str),
    Sgp4(String),
}

impl std::fmt::Display for IngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingLine(field) => write!(f, "missing TLE {field}"),
            Self::InvalidLine(field) => write!(f, "invalid TLE {field}"),
            Self::InvalidField(field) => write!(f, "invalid TLE field: {field}"),
            Self::Sgp4(message) => write!(f, "sgp4 propagation error: {message}"),
        }
    }
}

impl std::error::Error for IngestError {}

#[must_use]
pub fn celestrak_active_url() -> &'static str {
    "https://celestrak.org/NORAD/elements/gp.php?GROUP=active&FORMAT=tle"
}

pub fn parse_tle_records(input: &str) -> Result<Vec<TleRecord>, IngestError> {
    let tle_lines = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if tle_lines.is_empty() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    let mut index = 0;
    while index < tle_lines.len() {
        let name = tle_lines
            .get(index)
            .ok_or(IngestError::MissingLine("name"))?;
        let first_tle_line = tle_lines
            .get(index + 1)
            .ok_or(IngestError::MissingLine("line1"))?;
        let second_tle_line = tle_lines
            .get(index + 2)
            .ok_or(IngestError::MissingLine("line2"))?;

        if !first_tle_line.starts_with("1 ") {
            return Err(IngestError::InvalidLine("line1"));
        }
        if !second_tle_line.starts_with("2 ") {
            return Err(IngestError::InvalidLine("line2"));
        }

        records.push(TleRecord {
            name: (*name).to_string(),
            line1: (*first_tle_line).to_string(),
            line2: (*second_tle_line).to_string(),
        });
        index += 3;
    }

    Ok(records)
}

pub fn normalize_tle_record(record: &TleRecord) -> Result<NormalizedTleObservation, IngestError> {
    let satnum = field(&record.line1, 2, 7, "satellite_number")?.to_string();
    let state = propagate_tle_record(record, 0.0)?;
    let regime = regime_for_altitude(state.altitude_km());
    let object = SpaceObject {
        id: satnum.clone(),
        name: record.name.clone(),
        regime,
        state: state.clone(),
        tle: Some(TleEphemeris {
            line1: record.line1.clone(),
            line2: record.line2.clone(),
        }),
        behavior: BehaviorSignature {
            nominal_rf_mhz: None,
            historical_maneuvers_per_week: 0.0,
            station_keeping_expected: false,
        },
        risk: RiskProfile {
            operator: None,
            mission_class: MissionClass::Unknown,
            criticality: 0.4,
        },
    };
    let observation = Observation {
        object_id: satnum,
        observed_state: state,
        rf_mhz: None,
        maneuver_detected: false,
        source: ObservationSource::Catalog,
    };

    Ok(NormalizedTleObservation {
        object,
        observation,
    })
}

pub fn propagate_tle_record(
    record: &TleRecord,
    minutes_since_epoch: f64,
) -> Result<OrbitalState, IngestError> {
    let elements = sgp4::Elements::from_tle(
        Some(record.name.clone()),
        record.line1.as_bytes(),
        record.line2.as_bytes(),
    )
    .map_err(|error| IngestError::Sgp4(error.to_string()))?;
    let epoch_unix_seconds = elements.datetime.and_utc().timestamp();
    let constants = sgp4::Constants::from_elements(&elements)
        .map_err(|error| IngestError::Sgp4(error.to_string()))?;
    let prediction = constants
        .propagate(sgp4::MinutesSinceEpoch(minutes_since_epoch))
        .map_err(|error| IngestError::Sgp4(error.to_string()))?;

    Ok(OrbitalState {
        epoch_unix_seconds: epoch_unix_seconds
            .saturating_add(rounded_seconds_from_minutes(minutes_since_epoch)),
        position_km: vector3_from_prediction(prediction.position),
        velocity_km_s: vector3_from_prediction(prediction.velocity),
    })
}

fn regime_for_altitude(altitude_km: f64) -> OrbitRegime {
    if altitude_km < 2_000.0 {
        OrbitRegime::Leo
    } else if altitude_km < 35_000.0 {
        OrbitRegime::Meo
    } else if altitude_km < 37_000.0 {
        OrbitRegime::Geo
    } else {
        OrbitRegime::DeepSpace
    }
}

fn field<'a>(
    line: &'a str,
    start: usize,
    end: usize,
    name: &'static str,
) -> Result<&'a str, IngestError> {
    line.get(start..end)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(IngestError::InvalidField(name))
}

fn vector3_from_prediction(value: [f64; 3]) -> Vector3 {
    Vector3 {
        x: value[0],
        y: value[1],
        z: value[2],
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn rounded_seconds_from_minutes(minutes_since_epoch: f64) -> i64 {
    let seconds = (minutes_since_epoch * 60.0).round();
    if !seconds.is_finite() {
        return 0;
    }
    if seconds >= i64::MAX as f64 {
        i64::MAX
    } else if seconds <= i64::MIN as f64 {
        i64::MIN
    } else {
        seconds as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sss_core::{AnalyzeObjectRequest, AnticipationEngine, DigitalTwin, IntelligenceLayer};

    const CELESTRAK_ISS_SNAPSHOT: &str = "\
ISS (ZARYA)
1 25544U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9992
2 25544  51.6461 221.2784 0001413  89.1723 280.4612 15.49507896236008
";

    #[test]
    fn parses_celestrak_tle_snapshot() {
        let records = parse_tle_records(CELESTRAK_ISS_SNAPSHOT).expect("tle records");

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].name, "ISS (ZARYA)");
        assert!(records[0].line1.starts_with("1 25544"));
    }

    #[test]
    fn normalizes_tle_into_core_observation() {
        let record = parse_tle_records(CELESTRAK_ISS_SNAPSHOT)
            .expect("tle records")
            .remove(0);
        let normalized = normalize_tle_record(&record).expect("normalized tle");

        assert_eq!(normalized.object.id, "25544");
        assert_eq!(normalized.observation.object_id, "25544");
        assert_eq!(normalized.observation.source, ObservationSource::Catalog);
        assert!(matches!(normalized.object.regime, OrbitRegime::Leo));
        assert!(
            normalized
                .observation
                .observed_state
                .position_km
                .magnitude()
                > 6_500.0
        );
        assert!(
            normalized
                .observation
                .observed_state
                .velocity_km_s
                .magnitude()
                > 7.0
        );
        assert!(normalized.object.tle.is_some());
    }

    #[test]
    fn sgp4_propagates_tle_forward_in_time() {
        let record = parse_tle_records(CELESTRAK_ISS_SNAPSHOT)
            .expect("tle records")
            .remove(0);
        let epoch_state = propagate_tle_record(&record, 0.0).expect("epoch state");
        let future_state = propagate_tle_record(&record, 90.0).expect("future state");

        assert_eq!(
            future_state.epoch_unix_seconds - epoch_state.epoch_unix_seconds,
            5_400
        );
        assert!(
            future_state
                .position_km
                .distance_to(epoch_state.position_km)
                > 10.0
        );
        assert!(future_state.velocity_km_s.magnitude() > 7.0);
    }

    #[test]
    fn celestrak_tle_snapshot_feeds_core_intelligence_pipeline() {
        let record = parse_tle_records(CELESTRAK_ISS_SNAPSHOT)
            .expect("tle records")
            .remove(0);
        let normalized = normalize_tle_record(&record).expect("normalized tle");
        let mut layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![normalized.object]),
            AnticipationEngine::default(),
        );
        layer.record_observation(normalized.observation);

        let response = layer.analyze_object(AnalyzeObjectRequest {
            object_id: "25544".to_string(),
            timestamp_unix_seconds: None,
        });

        assert_eq!(response.object_id, "25544");
        assert_eq!(
            response.evidence_bundle.sources,
            vec![ObservationSource::Catalog]
        );
        assert_eq!(response.evidence_bundle.observations.len(), 1);
        assert!(!response.evidence_bundle.bundle_hash.is_empty());
    }
}
