# API reference

The public surface of `axum_guard_rs` `1.0.0`. The full crate documentation
is also in `src/lib.rs` (build it with `cargo doc --open`). Most behavior is
defined by [tower-guard-rs](https://github.com/rennf93/tower-guard-rs) and
re-exported here.

## Guard layer

### `with_guard`

```rust
use axum::Router;
use axum_guard_rs::{DetectConfig, with_guard};

let config = DetectConfig {
    max_content_length: 10_000,
    max_full_scan_bytes: 262_144,
    preserve_attack_patterns: true,
    semantic_threshold: 0.7,
    threat_score_threshold: 1.0,
};

let app: Router = Router::new().layer(with_guard(config));
```

`with_guard(config: DetectConfig) -> GuardLayer` builds the Guard layer for
an axum `Router`. Apply it with `Router::layer` (every registered route) or
`Router::route_layer` (routes only, skipping the fallback). Chain
`GuardLayer::with_body_cap` to change the body buffering cap:

```rust
let layer = axum_guard_rs::with_guard(axum_guard_rs::default_config())
    .with_body_cap(1_048_576);
let app = Router::new().layer(layer);
```

A body larger than the cap is rejected with `413 Payload Too Large` rather
than forwarded unscanned. A cap of `0` rejects every request that carries a
non-empty body.

## Configuration

### `default_config()`

Returns the reference default `DetectConfig`:

| Knob | Value |
|---|---|
| `max_content_length` | `10_000` |
| `max_full_scan_bytes` | `262_144` |
| `preserve_attack_patterns` | `true` |
| `semantic_threshold` | `0.7` |
| `threat_score_threshold` | `1.0` |

### `DetectConfig`

Re-exported from the engine (via `tower-guard-rs`). Fields:

| Field | Type | Meaning |
|---|---|---|
| `max_content_length` | `usize` | Semantic budget and truncation budget |
| `max_full_scan_bytes` | `usize` | Preprocessor full-scan cap (also the default body cap) |
| `preserve_attack_patterns` | `bool` | Keep attack patterns in the processed view |
| `semantic_threshold` | `f64` | Semantic analysis threshold |
| `threat_score_threshold` | `f64` | Threat score threshold for a verdict |

## Re-exports

From `tower-guard-rs`:

| Item | Kind |
|---|---|
| `GuardLayer` | The tower layer `with_guard` returns |
| `GuardService` | The tower service the layer produces |
| `GuardBody` | Body wrapper the service answers with |
| `BoxError` | Boxed error type used by the guard body |
| `default_config` | Reference default `DetectConfig` |
| `DetectConfig`, `DetectVerdict`, `Threat` | Engine types from `guard_core_engine::detect` |
| `BLOCKED_MESSAGE`, `OVERSIZE_MESSAGE`, `FAILURE_MESSAGE` | Refusal message bodies (`"Suspicious activity detected"`, `"Payload too large"`, `"Security check failed"`) |

## Behavior

### What it inspects

One engine call per request view:

| Request part | Engine context | Notes |
|---|---|---|
| Path | `url_path` | Skipped for `/` |
| Query string | `query_param` | Skipped when empty |
| Header values | `header` | Skips `sec-*` and the negotiation/routing headers |
| Body | `request_body` | Buffered first, capped |

The HTTP method is not scanned.

### Responses

| Situation | Status | Body |
|---|---|---|
| Engine flags a view | `403 Forbidden` | `{"detail":"Suspicious activity detected"}` |
| Body exceeds the cap | `413 Payload Too Large` | `{"detail":"Payload too large"}` |
| Body read error or engine panic | `500 Internal Server Error` | `{"detail":"Security check failed"}` |

The adapter is fail-secure: any failure to complete the security check
answers `500`, never an uninspected passthrough. Engine panics are caught
with `catch_unwind` (note that `panic = "abort"` in a release profile
disables that recovery).
