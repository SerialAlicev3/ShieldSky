# SSS SkyShield Architecture

`sss-skyshield` is an external vertical layer over `sss-core` for critical sites:

- solar plants
- data centers
- substations

It does not replace orbital intelligence. It consumes the existing SSS decision model and adds:

- site registry and protected zones
- site-specific exposure and impact semantics
- drone-defense-only response policies
- authorization gates and audit-oriented handoff

## Workspace Layout

```text
rust/crates/
  sss-core/            existing shared intelligence primitives
  sss-site-registry/   site, zone, asset, exposure-profile domain
  sss-skyshield/       threat domain, engines, policies, SSS-core mapping
  sss-response/        defensive response routing, authorization, audit
  sss-skyshield-api/   HTTP surface for the vertical
```

## Runtime Flow

```text
site sensors
  -> SkySignal
  -> AerialTrack
  -> SkyThreatAssessment
  -> SiteOperationalDecision
  -> SiteIncident
  -> SkyshieldCoreEnvelope
       -> ObservationFinding
       -> IntelligenceAssessment
       -> OperationalDecision
       -> EvidenceBundle
       -> ReplayManifest
       -> RankedEvent
  -> ResponsePolicy / AuthorizationGate
  -> third-party defensive handoff
```

## Safety Boundaries

- defensive response only
- no offensive logic
- no autonomous mitigation without gate
- no unaudited dispatch
- no direct hardware control in domain logic

## Current Status

The current implementation provides:

- registry structs for sites, zones, assets, and exposure
- skyshield domain structs for signals, tracks, threats, incidents, and decisions
- a first end-to-end assessment function that maps site context into `sss-core` types
- response-layer policy and authorization primitives
- a versioned HTTP API surface for site CRUD, overview, incidents, authorization gates, and response audit
- file-backed JSON persistence for sites, zones, assets, exposure profiles, incidents, and authorization gates
- an in-memory state mode for contract tests and fast local verification
- a first drone-defense HTTP pipeline at `POST /v1/sites/:id/drone-defense/assess`
- signal and track persistence through `POST /v1/sites/:id/signals` and `POST /v1/sites/:id/tracks`
- track lookup/classification through `GET /v1/sites/:id/tracks/:track_id` and `POST /v1/sites/:id/tracks/:track_id/classify`
- dispatch gating that blocks defensive handoff until the incident reaches `ResponseAuthorized`

By default, `sss-skyshield-api` persists to:

```text
data/sss-skyshield-state.json
```

Override this path with:

```text
SSS_SKYSHIELD_STATE_PATH=/path/to/skyshield-state.json
```

## Next Critical Block

The next implementation block is the mandatory drone-defense pipeline:

```text
SkySignal
  -> AerialTrack
  -> classification_engine
  -> behavior_engine
  -> impact_engine
  -> SiteIncident
  -> AuthorizationGate
  -> defensive handoff audit
```

This keeps the rule intact: detect, confirm, assess, decide, authorize, defend, audit.

The first version of this path is now active in the API, including persisted `SkySignal` and
`AerialTrack` history. The next deepening step is to connect RF/radar adapters instead of
accepting a single fused track payload.
