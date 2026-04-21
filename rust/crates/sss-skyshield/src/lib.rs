pub mod adapters;
pub mod application;
pub mod domain;
pub mod engines;
pub mod policies;

#[cfg(test)]
mod tests;

pub use application::assess_site_sky::{
    assess_site_sky, AssessSiteSkyInput, AssessSiteSkyOutput, SkyshieldCoreEnvelope,
};
pub use domain::decisions::{
    AuthorizationGate, EscalationPolicy, GateStatus, RecipientRole, ResponseAction, ResponseOption,
    ResponsePolicy, SiteOperationalDecision,
};
pub use domain::signals::{AerialTrack, ObjectType, SignalSource, SignalType, SkySignal, Vec3};
pub use domain::site::SiteContext;
pub use domain::threats::{
    IncidentStatus, IncidentType, Severity, SiteIncident, SkyThreatAssessment, ThreatType,
};
