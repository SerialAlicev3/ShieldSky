use serde::{Deserialize, Serialize};
use uuid::Uuid;

use sss_skyshield::{ResponseAction, ResponsePolicy, SiteIncident};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseRoute {
    AntiDrone,
    SiteSecurity,
    OperatorOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseDispatch {
    pub dispatch_id: Uuid,
    pub incident_id: Uuid,
    pub selected_action: ResponseAction,
    pub route: ResponseRoute,
    pub rationale: String,
}

#[must_use]
pub fn route_for(policy: &ResponsePolicy, incident: &SiteIncident) -> Option<ResponseDispatch> {
    let selected_action = *policy.allowed_actions.first()?;
    route_for_action(policy, incident, selected_action)
}

#[must_use]
pub fn route_for_action(
    policy: &ResponsePolicy,
    incident: &SiteIncident,
    selected_action: ResponseAction,
) -> Option<ResponseDispatch> {
    if !policy.allowed_actions.contains(&selected_action) {
        return None;
    }
    let route = match selected_action {
        ResponseAction::DispatchDroneIntercept
        | ResponseAction::DispatchDroneInspection
        | ResponseAction::DispatchDronePatrol => ResponseRoute::AntiDrone,
        ResponseAction::EscalateToSiteSecurity
        | ResponseAction::EscalateToSoc
        | ResponseAction::ActivatePerimeterHardening => ResponseRoute::SiteSecurity,
        ResponseAction::ObserveOnly => ResponseRoute::OperatorOnly,
    };
    Some(ResponseDispatch {
        dispatch_id: Uuid::new_v4(),
        incident_id: incident.incident_id,
        selected_action,
        route,
        rationale: "authorized defensive handoff route".to_string(),
    })
}
