from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class SssMilestone:
    name: str
    outcome: str
    implementation_surface: str


MVP_MILESTONES: tuple[SssMilestone, ...] = (
    SssMilestone(
        name="Digital twin core",
        outcome="Represent tracked space objects and predict simple future states.",
        implementation_surface="rust/crates/sss-core",
    ),
    SssMilestone(
        name="Behavior anomaly scoring",
        outcome="Compare observations against expected position, velocity, RF, and maneuver profile.",
        implementation_surface="rust/crates/sss-core",
    ),
    SssMilestone(
        name="Anticipation alert",
        outcome="Convert anomaly evidence into probability, severity, and operator recommendation.",
        implementation_surface="rust/crates/sss-core",
    ),
    SssMilestone(
        name="SSS Core API",
        outcome="Answer analyze-object, space-overview, and predict requests from normalized orbital data.",
        implementation_surface="rust/crates/sss-core::IntelligenceLayer",
    ),
    SssMilestone(
        name="Audit-ready intelligence contract",
        outcome="Separate findings, assessments, operational decisions, evidence bundles, and ranked events.",
        implementation_surface="rust/crates/sss-core",
    ),
    SssMilestone(
        name="Operator workflow",
        outcome="Use Claw as the agent harness for investigation, reporting, and guided analysis.",
        implementation_surface="claw CLI/runtime",
    ),
)


def render_sss_mvp_plan() -> str:
    lines = ["Serial Alice Sky MVP Plan", ""]
    for index, milestone in enumerate(MVP_MILESTONES, start=1):
        lines.append(f"{index}. {milestone.name}")
        lines.append(f"   Outcome: {milestone.outcome}")
        lines.append(f"   Surface: {milestone.implementation_surface}")
    return "\n".join(lines)
