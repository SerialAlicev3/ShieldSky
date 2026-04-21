# Serial Alice API Reuse For SSS

Source reviewed: `C:/Users/Nelso/Desktop/implementacoes/serial-aliceapi122.zip`.

The reusable value is not the energy-certification domain. The reusable value is the trust architecture around evidence, API boundaries, replay, and operational intelligence.

## Reuse Directly

1. Evidence event model

Serial Alice API has a strong append-only event language: measurement, validation, interpretation, policy, decision, action, outcome, orchestration, and intelligence. For SSS this maps cleanly to orbital intelligence:

- observation: normalized orbital/RF/catalog input
- validation: cross-source confirmation
- interpretation: anomaly or behavior assessment
- policy: recipient/escalation policy in force
- assessment: intelligence judgment
- decision: operational recommendation
- notification: who was told and why
- outcome: what happened after the decision

2. Evidence strength

The API makes evidence strength explicit. This is critical for SSS because a radar observation, a catalog update, an ML inference, and an operator note must not be treated as equal evidence.

The first Rust port is now in `sss-core` as `EvidenceStrength`.

3. Evidence bundle export

The API has `/v2/evidence/{id}/bundle`, designed for external audits. SSS should mirror this idea as `/v1/evidence/{assessment_id}/bundle` once `sss-api` exists.

4. Replay manifest

The API has a replay manifest that fixes module versions at decision time. This maps directly to SSS questions such as: why did object X score 0.82 on a specific day with a specific model version?

The first Rust port is now in `sss-core` as `ReplayManifest` and `replay_manifest_for`.

5. API envelopes

The API's `AsyncCreatedEnvelope`, `StatusEnvelope`, `ErrorEnvelope`, `VerificationEnvelope`, and `ListEnvelope` are worth reusing conceptually for `sss-api`. They make the external API predictable and easier to integrate.

6. Insights catalogue

The API's insights module uses formal machine-readable insight types, severities, actions, resource IDs, and dismissal. SSS should use the same style for operator console events:

- behavior_shift_detected
- predicted_close_approach
- coordination_pattern_suspected
- evidence_gap
- validation_needed

## Reuse With Adaptation

- Anomaly detector threshold pattern: reuse the stateless detector style, not the GPU-specific thresholds.
- Confidence calibrator: adapt cohorts from `(workload_class, hardware_family)` to `(orbit_regime, object_class, source_mix)`.
- Node health manager: adapt the quarantine/degraded model into `watch`, `warning`, `critical`, and `institutional_escalation`.
- Governance and policy versioning: adapt to recipient policy, model policy, and replay policy.
- Rate limiting and tenant enforcement: useful later for SaaS and enterprise deployments.

## Do Not Reuse

- Energy/carbon-specific certificate vocabulary.
- Hardware/GPU/PDU adapters.
- Browser temp data, node modules, caches, pycache, and worktree artifacts.
- Greenwashing-specific logic except as inspiration for "unsupported claim" detection.

## SSS Boundary Proposal

- `sss-core`: Rust domain intelligence, evidence events, bundles, assessment, decision, replay manifest.
- `sss-api`: HTTP API with envelopes and versioned endpoints.
- `sss-ingest`: external adapters such as CelesTrak, Space-Track, commercial feeds, RF feeds.
- `sss-events`: serialized event contracts for replay, timelines, notifications, and audit export.

## First Extract Already Applied

`sss-core` now contains:

- `EvidenceStrength`
- `IntelligenceEventType`
- `IntelligenceEvent`
- `EvidenceBundle.events`
- `EvidenceBundle.bundle_hash`
- `ReplayManifest`
- `replay_manifest_for`

This lets SSS treat intelligence as replayable, auditable evidence rather than just a JSON answer.
