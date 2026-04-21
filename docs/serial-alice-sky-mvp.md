# Serial Alice Sky MVP

Serial Alice Sky starts as an anticipatory intelligence layer for orbital events. It does not compete with sensor networks, radar systems, telescopes, or catalog providers. It turns normalized orbital data into behavior, anticipation, risk, and decision support.

## MVP Architecture

```text
sensor networks / catalogs / commercial feeds
        |
        v
observation normalization
        |
        v
raw orbital data already collected elsewhere
        |
        v
SSS intelligence layer
        |
        v
insight / prediction / decision / distribution
        |
        v
Claw-guided operator workflow
```

## Core Questions

SSS must answer five questions better than a passive tracking stack:

1. What changed?
2. Is this normal?
3. What is likely to happen next?
4. Is it relevant?
5. Who needs to know?

## Current Code

- `rust/crates/sss-core` contains the first Rust intelligence core.
- `DigitalTwin` stores active objects and predicts simple future states.
- `AnomalyEngine` compares observations against the twin baseline.
- `AnticipationEngine` converts anomaly evidence into probability, severity, and recommendation.
- `IntelligenceLayer` exposes the first API-shaped product surface.
- `ObservationFinding`, `IntelligenceAssessment`, and `OperationalDecision` separate what was seen, what it means, and what to do.
- `EvidenceBundle` records the observations, sources, derived features, and confidence contributors used by an assessment.
- `EvidenceStrength`, `IntelligenceEvent`, and `ReplayManifest` bring over the audit/replay pattern from the existing Serial Alice API.
- `RankedEvent` makes the operator workflow event-first instead of object-first.
- `rust/crates/sss-api` contains the first Axum HTTP service around the core.
- `rust/crates/sss-ingest` parses CelesTrak/NORAD-style TLE snapshots and normalizes them into `SpaceObject` and `Observation`.
- `rust/crates/sss-storage` persists space objects, observations, evidence bundles, replay manifests, assessments, ranked events, ingest batches, and request/response logs in SQLite.
- `examples/mvp.rs` runs a synthetic end-to-end scenario.

## First Service Surface

The current implementation now has both an in-process Rust API and a thin HTTP service. Product logic stays in `sss-core`; `sss-api` handles versioned HTTP routing, JSON envelopes, request IDs, tracing, and SQLite-backed evidence/replay storage.

```text
GET  /v1/health                 -> service health
GET  /v1/version                -> API and model version
GET  /v1/briefing/apod          -> NASA Astronomy Picture of the Day briefing
GET  /v1/briefing/neows         -> NASA Near Earth Object risk briefing summary
GET  /v1/briefing/neows/feed    -> NASA Near Earth Object detailed feed briefing
POST /v1/ingest/tle             -> CelesTrak/NORAD-style TLE ingest
POST /v1/ingest/celestrak-active -> live CelesTrak active feed ingest
GET  /v1/ingest/batches         -> recent ingest batch history
GET  /v1/ingest/status          -> freshness/status for one ingest source
POST /v1/analyze-object         -> IntelligenceLayer::analyze_object
POST /v1/space-overview         -> IntelligenceLayer::space_overview
POST /v1/predict                -> IntelligenceLayer::predict
POST /v1/notifications/dispatch -> analyze + deliver decision notifications to webhook
POST /v1/events/dispatch        -> dispatch notifications directly from a ranked event
GET  /v1/notifications          -> notification delivery audit trail
GET  /v1/events/timeline        -> future event buckets for operator console
GET  /v1/events                 -> persisted ranked events
GET  /v1/objects/{id}/assessments -> persisted assessment history
GET  /v1/objects/{id}/predictions -> persisted prediction snapshots
GET  /v1/objects/{id}/close-approaches -> native SSS close-approach briefing
GET  /v1/objects/{id}/timeline  -> future projection checkpoints
GET  /v1/evidence/{bundle_hash} -> stored EvidenceBundle
GET  /v1/replay/{manifest_hash} -> stored ReplayManifest
GET  /v1/replay/{manifest_hash}/execute -> replayed assessment comparison
```

`/analyze-object` resolves one object, compares the latest normalized observation with its behavioral twin, and returns status, anomaly score, confidence, prediction, explanation, risk, severity, and intended recipients.

`/ingest/tle` receives a CelesTrak/NORAD-style TLE payload, normalizes records into SSS objects and observations, upserts objects into the digital twin, records latest observations, and persists the ingest batch. Identical payloads from the same source are deduplicated instead of being reprocessed.

`/ingest/celestrak-active` fetches the live CelesTrak active feed and runs the same TLE normalization and persistence pipeline. If the upstream source is unavailable, the API returns `source_unavailable`.

`/ingest/batches` exposes the most recent ingest runs so an operator can inspect source, record counts, object coverage, and raw payload provenance without opening the database directly.

`/ingest/status?source=celestrak-active` returns the most recent ingest request id, object coverage, and freshness in seconds for one source.

`/space-overview` ranks recent events by risk for a selected time window.

`/predict` returns the forward-looking probability model for one object and horizon.

`/briefing/apod?date=YYYY-MM-DD` fetches NASA's Astronomy Picture of the Day and returns it as a lightweight briefing payload for the operator console or daily intelligence digest.

`/briefing/neows?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD` fetches NASA NeoWS close-approach data and condenses it into a risk briefing with hazardous count, approach windows, miss distance, velocity, size estimates, heuristic priority scoring, and top-priority objects for operator briefing.

`/briefing/neows/feed?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD` exposes the same period as a richer ranked feed so the console or downstream consumers can inspect every object in priority order.

`/events` and `/objects/{id}/assessments` give an operator direct access to prioritized events and the historical intelligence assessments for one object. `/events` now supports `object_id`, `event_type`, `future_only`, `after_unix_seconds`, and `before_unix_seconds` so the operator console can ask for only future events inside a relevant window.

`/events/timeline?horizon=72` returns the same ranked events arranged into operator-facing future buckets such as `0-6h`, `6-24h`, and `24h+`, ordered by temporal urgency first and risk second inside each bucket. `reference_unix_seconds` can pin the window for replay, demos, or historical inspection instead of using wall-clock now.

`/space-overview` and persisted `/events` entries now carry `target_epoch_unix_seconds`, so event ranking can take temporal urgency into account and the UI can distinguish predicted events that need attention soon from those that are only high risk in the abstract.

`/space-overview` now also synthesizes native SSS close-approach events from pairwise projected object states, so predicted encounters can be ranked, persisted, surfaced in `/events`, and shown in `/events/timeline` without depending on an object-detail call first.

`/notifications/dispatch` turns an analysis into real outbound delivery over a configured webhook. Each recipient notification becomes one webhook POST, retries according to policy when delivery fails, and `/notifications` exposes the persisted delivery history with request id, recipient, target, status, response code, attempt number, and notification reason.

`/events/dispatch` lets the operator dispatch from a persisted ranked event id after `/space-overview` or `/events` has already identified the priority item. The service resolves the event, reuses the hardened analysis/notification path for that object, and keeps the dispatch auditable without forcing a separate manual analyze step.

`/objects/{id}/timeline?horizon=72` returns projection checkpoints for the object at `t+1h`, `t+6h`, `t+24h`, and the requested horizon. Each checkpoint includes target epoch, propagation model, projected orbital state, and a short operator summary.

`/objects/{id}/predictions?limit=10` returns persisted prediction snapshots for one object, including the generating endpoint, model version, propagation model, base epoch, and the exact prediction payload captured at that moment.

`/objects/{id}/close-approaches?horizon=72&threshold_km=250` scans the projected states of neighboring objects across the selected horizon and returns the highest-priority native SSS close approaches, including miss distance, target epoch, propagation model, pair states, and a heuristic priority score for operator triage.

`/replay/{manifest_hash}/execute` resolves the manifest, finds the matching evidence bundle, recomputes the assessment and operational decision from stored evidence, and returns the original vs replayed outputs together with an explicit diff for changed fields, score deltas, recipients, and escalation rules.

When `analyze-object` detects a pairwise projected close approach for the selected object, the operational decision now upgrades the primary prediction to `CloseApproach`, adds a dedicated escalation rule for orbital coordination, and emits recipient-specific notification reasons that explain the projected encounter.

The first real-data-capable path is now:

```text
CelesTrak-style TLE snapshot or live CelesTrak active feed
    -> POST /v1/ingest/tle or POST /v1/ingest/celestrak-active
    -> normalized SpaceObject + Observation
    -> digital twin upsert
    -> POST /v1/analyze-object
    -> API response
    -> SQLite evidence/replay persistence
    -> evidence retrieval
    -> replay manifest retrieval
    -> replay execution
```

By default, `sss-api` stores data at `data/sss-api.sqlite`. Set `SSS_STORAGE_PATH` to use a different SQLite database path. The service creates the parent directory when needed.

The TLE normalization path now uses SGP4 propagation rather than the previous simplified orbital approximation. `SpaceObject` instances created from TLE ingest retain their original TLE lines so the digital twin can project future state with the same SGP4 model.

`/predict` now carries projected state, propagation model, observation epoch, and trajectory deltas so future behavior can be explained as a concrete orbital projection rather than only a behavioral label.

Both `/predict` and `/analyze-object` now persist `PredictionSnapshot` records so future expectations can be inspected directly without re-parsing generic request logs.

The operator console now overlays a native close-approach layer on the Cesium globe, drawing projected encounter lines and a counterpart marker for the most urgent approaches alongside the existing SSS event and NEO-context layers.

The API service can also run a background live-ingest worker. Set `SSS_CELESTRAK_ACTIVE_POLL_SECONDS` to enable periodic polling of the active CelesTrak feed, and optionally set `SSS_CELESTRAK_ACTIVE_RETRY_SECONDS` to control retry delay after a failed fetch.

Set `SSS_WEBHOOK_ENDPOINT` to enable outbound notification delivery from `/v1/notifications/dispatch`. `SSS_WEBHOOK_RETRY_ATTEMPTS` and `SSS_WEBHOOK_RETRY_DELAY_MS` control retry count and backoff between attempts.

Set `SSS_NASA_API_KEY` to use your own NASA API key for `/v1/briefing/apod`. If not set, the service falls back to `DEMO_KEY`. `SSS_NASA_API_BASE_URL` can override the NASA base URL for testing or proxying.

On startup, `sss-api` hydrates the digital twin from persisted `SpaceObject` records and restores the latest persisted `Observation` per object, so ingested objects can still be analyzed after a restart.

## Audit Model

Every analysis should be replayable later. The MVP response already carries:

- `finding`: observed evidence and derived anomaly features.
- `assessment`: behavior label, confidence, prediction, rule hits, model version, and decision version.
- `decision`: severity, risk score, recommendation, recipient policy, escalation rules, and notification reasons.
- `evidence_bundle`: source observations and confidence contributors.
- `replay_manifest`: derived from the bundle when a caller needs to freeze model and policy versions for replay.

This is the foundation for answering: "why was the score different with model X on day Y?"

## Service Boundaries

Keep the product boundaries clean as the repo grows:

- `sss-core`: domain model, intelligence logic, assessments, decisions, event ranking.
- `sss-api`: HTTP/gRPC surface such as `/v1/analyze-object`.
- `sss-ingest`: CelesTrak, Space-Track, commercial feeds, and normalization adapters.
- `sss-events`: event contracts for timelines, alerts, replay, and downstream integrations.

## Next Build Targets

1. Add delivery retry/backoff and secondary channels beyond webhook.
2. Add explicit API errors for object lookup, insufficient evidence, unavailable prediction, and validation failures.
3. Build a Three.js dashboard that renders object state, anomaly trails, prediction snapshots, and future event timelines.
4. Expand ingest adapters beyond CelesTrak to Space-Track and stored capture replay.
5. Add model/policy version comparisons directly into operator workflows.

## Claw Role

Claw remains the agent layer around the product. It can run investigations, generate reports, call tools, inspect telemetry, and coordinate future skills/plugins for space intelligence workflows. The SSS product logic should stay isolated from the Claw runtime so it can later become its own service.
