use serde::{Deserialize, Serialize};
use sss_site_registry::{Criticality, Site, SiteType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherNowSignal {
    pub observed_at_unix_seconds: i64,
    pub wind_kph: f64,
    pub hail_probability: f64,
    pub irradiance_drop_ratio: f64,
    pub cloud_cover_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteForecastInput {
    pub site: Site,
    pub generated_at_unix_seconds: i64,
    pub horizon_hours: u32,
    pub observation_confidence: f64,
    pub latest_risk: f64,
    pub weather_now: WeatherNowSignal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteForecastPoint {
    pub offset_hours: u32,
    pub target_epoch_unix_seconds: i64,
    pub modeled_solar_generation_kw: f64,
    pub modeled_load_kw: f64,
    pub modeled_price_index: f64,
    pub reserve_stress_index: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolarForecastResponse {
    pub site_id: String,
    pub site_name: String,
    pub generated_at_unix_seconds: i64,
    pub horizon_hours: u32,
    pub current_irradiance_drop_ratio: f64,
    pub confidence: f64,
    pub points: Vec<SiteForecastPoint>,
    pub narrative: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadForecastResponse {
    pub site_id: String,
    pub site_name: String,
    pub generated_at_unix_seconds: i64,
    pub horizon_hours: u32,
    pub confidence: f64,
    pub points: Vec<SiteForecastPoint>,
    pub narrative: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioForecastResponse {
    pub site_id: String,
    pub site_name: String,
    pub generated_at_unix_seconds: i64,
    pub horizon_hours: u32,
    pub confidence: f64,
    pub points: Vec<SiteForecastPoint>,
    pub recommendation_window_start_unix_seconds: i64,
    pub recommendation_window_end_unix_seconds: i64,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchestratorRecommendationRequest {
    pub horizon_hours: Option<u32>,
    pub battery_state_of_charge: Option<f64>,
    pub reserve_margin_ratio: Option<f64>,
    pub price_signal_bias: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestratorAction {
    ChargeBatteries,
    HoldReserve,
    DispatchReserve,
    LowerPrices,
    RaisePrices,
    MonitorOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchestratorRecommendation {
    pub site_id: String,
    pub site_name: String,
    pub generated_at_unix_seconds: i64,
    pub horizon_hours: u32,
    pub action: OrchestratorAction,
    pub confidence: f64,
    pub recommended_window_start_unix_seconds: i64,
    pub recommended_window_end_unix_seconds: i64,
    pub expected_benefit: String,
    pub operational_reason: String,
    pub safeguards: Vec<String>,
    pub scenario: ScenarioForecastResponse,
}

#[must_use]
pub fn solar_forecast(input: &SiteForecastInput) -> SolarForecastResponse {
    let points = forecast_points(input);
    SolarForecastResponse {
        site_id: input.site.site_id.to_string(),
        site_name: input.site.name.clone(),
        generated_at_unix_seconds: input.generated_at_unix_seconds,
        horizon_hours: input.horizon_hours,
        current_irradiance_drop_ratio: input.weather_now.irradiance_drop_ratio,
        confidence: aggregate_confidence(&points, input.observation_confidence),
        narrative: format!(
            "Modeled solar outlook over the next {}h for {:?}: irradiance pressure is {:.0}% with wind at {:.0} kph.",
            input.horizon_hours,
            input.site.site_type,
            input.weather_now.irradiance_drop_ratio * 100.0,
            input.weather_now.wind_kph
        ),
        points,
    }
}

#[must_use]
pub fn load_forecast(input: &SiteForecastInput) -> LoadForecastResponse {
    let points = forecast_points(input);
    LoadForecastResponse {
        site_id: input.site.site_id.to_string(),
        site_name: input.site.name.clone(),
        generated_at_unix_seconds: input.generated_at_unix_seconds,
        horizon_hours: input.horizon_hours,
        confidence: aggregate_confidence(&points, input.observation_confidence),
        narrative: format!(
            "Modeled load outlook over the next {}h for {:?}: baseline demand reacts to weather pressure and current site risk at {:.0}%.",
            input.horizon_hours,
            input.site.site_type,
            input.latest_risk * 100.0
        ),
        points,
    }
}

#[must_use]
pub fn scenario_forecast(input: &SiteForecastInput) -> ScenarioForecastResponse {
    let points = forecast_points(input);
    let most_stressed = points
        .iter()
        .max_by(|left, right| {
            left.reserve_stress_index
                .total_cmp(&right.reserve_stress_index)
                .then_with(|| {
                    right
                        .modeled_price_index
                        .total_cmp(&left.modeled_price_index)
                })
        })
        .cloned()
        .unwrap_or_else(|| points[0].clone());
    ScenarioForecastResponse {
        site_id: input.site.site_id.to_string(),
        site_name: input.site.name.clone(),
        generated_at_unix_seconds: input.generated_at_unix_seconds,
        horizon_hours: input.horizon_hours,
        confidence: aggregate_confidence(&points, input.observation_confidence),
        recommendation_window_start_unix_seconds: most_stressed.target_epoch_unix_seconds,
        recommendation_window_end_unix_seconds: most_stressed
            .target_epoch_unix_seconds
            .saturating_add(3_600),
        summary: format!(
            "Peak stress is projected around t+{}h with reserve pressure {:.0}% and price index {:.0}%.",
            most_stressed.offset_hours,
            most_stressed.reserve_stress_index * 100.0,
            most_stressed.modeled_price_index * 100.0
        ),
        points,
    }
}

#[must_use]
pub fn recommend_action(
    input: &SiteForecastInput,
    request: &OrchestratorRecommendationRequest,
) -> OrchestratorRecommendation {
    let scenario = scenario_forecast(input);
    let battery_soc = request
        .battery_state_of_charge
        .unwrap_or(0.45)
        .clamp(0.0, 1.0);
    let reserve_margin = request.reserve_margin_ratio.unwrap_or(0.18).clamp(0.0, 1.0);
    let price_bias = request.price_signal_bias.unwrap_or(0.0).clamp(-1.0, 1.0);
    let peak = scenario
        .points
        .iter()
        .max_by(|left, right| {
            left.reserve_stress_index
                .total_cmp(&right.reserve_stress_index)
        })
        .cloned()
        .unwrap_or_else(|| scenario.points[0].clone());
    let (action, confidence, expected_benefit, operational_reason) =
        recommendation_decision(&peak, battery_soc, reserve_margin, price_bias);

    OrchestratorRecommendation {
        site_id: input.site.site_id.to_string(),
        site_name: input.site.name.clone(),
        generated_at_unix_seconds: input.generated_at_unix_seconds,
        horizon_hours: input.horizon_hours,
        action,
        confidence: (confidence
            * aggregate_confidence(&scenario.points, input.observation_confidence))
        .clamp(0.0, 0.99),
        recommended_window_start_unix_seconds: scenario.recommendation_window_start_unix_seconds,
        recommended_window_end_unix_seconds: scenario.recommendation_window_end_unix_seconds,
        expected_benefit,
        operational_reason,
        safeguards: vec![
            "Human approval remains required before dispatching physical reserves.".to_string(),
            "Re-run forecast when weather, site risk, or price inputs drift materially."
                .to_string(),
        ],
        scenario,
    }
}

fn recommendation_decision(
    peak: &SiteForecastPoint,
    battery_soc: f64,
    reserve_margin: f64,
    price_bias: f64,
) -> (OrchestratorAction, f64, String, String) {
    if peak.reserve_stress_index >= 0.75 && battery_soc < 0.70 {
        (
            OrchestratorAction::ChargeBatteries,
            0.82,
            "Increase stored energy before the projected stress window.".to_string(),
            format!(
                "Stress is projected to spike to {:.0}% while state of charge is only {:.0}%.",
                peak.reserve_stress_index * 100.0,
                battery_soc * 100.0
            ),
        )
    } else if peak.modeled_price_index >= 0.72 && reserve_margin < 0.25 {
        (
            OrchestratorAction::HoldReserve,
            0.78,
            "Preserve reserve capacity for the expensive or constrained window ahead.".to_string(),
            format!(
                "Modeled price index reaches {:.0}% with reserve margin at {:.0}%.",
                peak.modeled_price_index * 100.0,
                reserve_margin * 100.0
            ),
        )
    } else if peak.modeled_solar_generation_kw > peak.modeled_load_kw * 1.35 && price_bias <= 0.1 {
        (
            OrchestratorAction::LowerPrices,
            0.70,
            "Use expected excess generation to pull flexible demand into the window.".to_string(),
            format!(
                "Modeled generation exceeds load by {:.0}%.",
                ((peak.modeled_solar_generation_kw / peak.modeled_load_kw) - 1.0) * 100.0
            ),
        )
    } else if peak.reserve_stress_index >= 0.60 && battery_soc >= 0.72 {
        (
            OrchestratorAction::DispatchReserve,
            0.74,
            "Prepare targeted reserve dispatch before the constrained interval begins.".to_string(),
            format!(
                "Reserve stress is {:.0}% and batteries are already charged at {:.0}%.",
                peak.reserve_stress_index * 100.0,
                battery_soc * 100.0
            ),
        )
    } else if peak.modeled_price_index <= 0.32 && price_bias > 0.2 {
        (
            OrchestratorAction::RaisePrices,
            0.62,
            "Avoid pulling demand into a low-value interval when price strategy is already upward."
                .to_string(),
            format!(
                "Modeled price index is only {:.0}% while external price bias is positive.",
                peak.modeled_price_index * 100.0
            ),
        )
    } else {
        (
            OrchestratorAction::MonitorOnly,
            0.58,
            "No pre-emptive move dominates the current scenario envelope.".to_string(),
            "Forecast pressure remains inside normal operating bounds.".to_string(),
        )
    }
}

fn forecast_points(input: &SiteForecastInput) -> Vec<SiteForecastPoint> {
    let capacity_kw = modeled_capacity_kw(input.site.site_type, input.site.criticality);
    let base_load_kw = modeled_base_load_kw(input.site.site_type, input.site.criticality);
    let horizon = input.horizon_hours.max(1);
    (1..=horizon)
        .map(|offset_hours| {
            let target_epoch_unix_seconds = input
                .generated_at_unix_seconds
                .saturating_add(i64::from(offset_hours) * 3_600);
            let diurnal = solar_shape(offset_hours);
            let weather_drag = 1.0
                - (input.weather_now.irradiance_drop_ratio * 0.55
                    + input.weather_now.hail_probability * 0.10);
            let wind_drag = 1.0 - (input.weather_now.wind_kph / 180.0).clamp(0.0, 0.35);
            let risk_drag = 1.0 - (input.latest_risk * 0.18);
            let modeled_solar_generation_kw = match input.site.site_type {
                SiteType::SolarPlant => {
                    capacity_kw * diurnal * weather_drag * wind_drag * risk_drag
                }
                SiteType::DataCenter => capacity_kw * diurnal * weather_drag * 0.35,
                SiteType::Substation => capacity_kw * diurnal * weather_drag * 0.22,
            }
            .max(0.0);

            let load_ramp = load_shape(offset_hours, input.site.site_type);
            let weather_load_pressure = 1.0
                + (input.weather_now.cloud_cover_ratio * 0.04)
                + (input.weather_now.wind_kph / 300.0);
            let risk_load_pressure = 1.0 + input.latest_risk * 0.12;
            let modeled_load_kw =
                (base_load_kw * load_ramp * weather_load_pressure * risk_load_pressure).max(0.0);

            let price_index = (((modeled_load_kw / (modeled_solar_generation_kw + 1.0)) * 0.45)
                + input.latest_risk * 0.35
                + (f64::from(offset_hours) / f64::from(horizon)) * 0.20)
                .clamp(0.0, 1.0);
            let reserve_stress_index =
                ((modeled_load_kw / (modeled_solar_generation_kw + base_load_kw * 0.4)) * 0.55
                    + input.weather_now.hail_probability * 0.20
                    + input.latest_risk * 0.25)
                    .clamp(0.0, 1.0);
            let confidence = (input.observation_confidence
                * (1.0 - (f64::from(offset_hours.saturating_sub(1)) * 0.018)))
                .clamp(0.25, 0.99);

            SiteForecastPoint {
                offset_hours,
                target_epoch_unix_seconds,
                modeled_solar_generation_kw,
                modeled_load_kw,
                modeled_price_index: price_index,
                reserve_stress_index,
                confidence,
            }
        })
        .collect()
}

fn modeled_capacity_kw(site_type: SiteType, criticality: Criticality) -> f64 {
    let base = match site_type {
        SiteType::SolarPlant => 60_000.0,
        SiteType::DataCenter => 8_000.0,
        SiteType::Substation => 15_000.0,
    };
    base * criticality_multiplier(criticality)
}

fn modeled_base_load_kw(site_type: SiteType, criticality: Criticality) -> f64 {
    let base = match site_type {
        SiteType::SolarPlant => 4_500.0,
        SiteType::DataCenter => 12_000.0,
        SiteType::Substation => 7_500.0,
    };
    base * criticality_multiplier(criticality)
}

fn criticality_multiplier(criticality: Criticality) -> f64 {
    match criticality {
        Criticality::Low => 0.85,
        Criticality::Medium => 1.0,
        Criticality::High => 1.18,
        Criticality::Critical => 1.35,
    }
}

fn solar_shape(offset_hours: u32) -> f64 {
    let hour = f64::from(offset_hours % 24);
    let daylight = ((hour - 6.0) / 12.0 * std::f64::consts::PI).sin().max(0.0);
    daylight.powf(1.2)
}

fn load_shape(offset_hours: u32, site_type: SiteType) -> f64 {
    let hour = f64::from(offset_hours % 24);
    match site_type {
        SiteType::SolarPlant => 0.82 + ((hour / 24.0) * std::f64::consts::TAU).sin().abs() * 0.30,
        SiteType::DataCenter => 0.96 + ((hour / 24.0) * std::f64::consts::TAU).cos().abs() * 0.12,
        SiteType::Substation => 0.88 + ((hour / 24.0) * std::f64::consts::TAU).sin().abs() * 0.22,
    }
}

fn aggregate_confidence(points: &[SiteForecastPoint], base_confidence: f64) -> f64 {
    let mean = if points.is_empty() {
        base_confidence
    } else {
        let point_count = u32::try_from(points.len()).unwrap_or(u32::MAX);
        points.iter().map(|point| point.confidence).sum::<f64>() / f64::from(point_count)
    };
    ((mean * 0.75) + (base_confidence * 0.25)).clamp(0.0, 0.99)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sss_site_registry::Site;
    use uuid::Uuid;

    fn sample_input(site_type: SiteType) -> SiteForecastInput {
        SiteForecastInput {
            site: Site {
                site_id: Uuid::new_v4(),
                name: "Forecast Test Site".to_string(),
                site_type,
                latitude: 38.5,
                longitude: -7.9,
                elevation_m: 120.0,
                timezone: "Europe/Lisbon".to_string(),
                country_code: "PT".to_string(),
                criticality: Criticality::High,
                operator_name: "Ops".to_string(),
            },
            generated_at_unix_seconds: 1_777_000_000,
            horizon_hours: 24,
            observation_confidence: 0.76,
            latest_risk: 0.41,
            weather_now: WeatherNowSignal {
                observed_at_unix_seconds: 1_777_000_000,
                wind_kph: 36.0,
                hail_probability: 0.20,
                irradiance_drop_ratio: 0.30,
                cloud_cover_ratio: 0.42,
            },
        }
    }

    #[test]
    fn solar_forecast_produces_points() {
        let forecast = solar_forecast(&sample_input(SiteType::SolarPlant));
        assert_eq!(forecast.points.len(), 24);
        assert!(forecast.confidence > 0.0);
    }

    #[test]
    fn recommendation_prefers_preemptive_action_under_stress() {
        let recommendation = recommend_action(
            &sample_input(SiteType::DataCenter),
            &OrchestratorRecommendationRequest {
                horizon_hours: Some(24),
                battery_state_of_charge: Some(0.22),
                reserve_margin_ratio: Some(0.10),
                price_signal_bias: Some(0.0),
            },
        );
        assert_ne!(recommendation.action, OrchestratorAction::MonitorOnly);
    }
}
