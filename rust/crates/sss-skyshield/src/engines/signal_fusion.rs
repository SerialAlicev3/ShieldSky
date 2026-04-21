use crate::SkySignal;

#[must_use]
pub fn fusion_summary(signals: &[SkySignal]) -> String {
    let parts = signals
        .iter()
        .map(|signal| format!("{:?}:{:?}", signal.source, signal.signal_type))
        .collect::<Vec<_>>();
    if parts.is_empty() {
        "no fused signals available".to_string()
    } else {
        format!("fused signals -> {}", parts.join(", "))
    }
}
