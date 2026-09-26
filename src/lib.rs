//! # axum-guard-rs
//!
//! Application-layer security middleware for [axum](https://github.com/tokio-rs/axum),
//! powered by the [guard-core-rs](https://github.com/rennf93/guard-core-rs)
//! detection engine. Part of the [Guard ecosystem](https://github.com/rennf93).
//!
//! ## Status: implemented (v0.1.0)
//!
//! [`with_guard`] returns the generic [`GuardLayer`] from
//! [`tower-guard-rs`](https://github.com/rennf93/tower-guard-rs), which is
//! already axum-shaped: axum routers are `tower` services, so
//! [`Router::layer`](axum::Router::layer) is the whole integration. This crate
//! is the thin axum-facing surface: the `with_guard` constructor, the
//! re-exports an axum application needs, and the axum-specific tests that pin
//! the behavior against `axum::body::Body`.
//!
//! Per the ecosystem boundary rules this crate contains no security logic:
//! detection comes from the engine, request screening from `tower-guard-rs`.
//!
//! ## What the guard does
//!
//! One engine call per request view (`url_path` for the path, `query_param`
//! for the query string, `header` for non-excluded header values,
//! `request_body` for the buffered body). Bodies are buffered up to a bounded
//! cap (262 144 bytes by default) and oversized bodies are rejected with `413`
//! rather than forwarded unscanned. The adapter is fail-secure: a body read
//! error or an engine panic answers `500`, never an uninspected passthrough.
//! See the [`tower-guard-rs`](https://github.com/rennf93/tower-guard-rs)
//! documentation for the full behavior tables, response shapes, and the
//! header exclusion list.
//!
//! ## Example
//!
//! ```
//! use axum::body::Body;
//! use axum::routing::get;
//! use axum::{Router, http::Request};
//! use tower::ServiceExt;
//!
//! # let runtime = tokio::runtime::Builder::new_current_thread()
//! #     .enable_all()
//! #     .build()
//! #     .unwrap();
//! # runtime.block_on(async {
//! let app = Router::new()
//!     .route("/hello", get(|| async { "hello" }))
//!     .layer(axum_guard_rs::with_guard(axum_guard_rs::default_config()));
//!
//! let request = Request::builder()
//!     .uri("/hello")
//!     .body(Body::empty())
//!     .unwrap();
//! let response = app.oneshot(request).await.unwrap();
//! assert_eq!(response.status(), 200);
//!
//! // Attack traffic is blocked by the engine.
//! let request = Request::builder()
//!     .uri("/hello?cmd=$(whoami)")
//!     .body(Body::empty())
//!     .unwrap();
//! let response = Router::new()
//!     .route("/hello", get(|| async { "hello" }))
//!     .layer(axum_guard_rs::with_guard(axum_guard_rs::default_config()))
//!     .oneshot(request)
//!     .await
//!     .unwrap();
//! assert_eq!(response.status(), 403);
//! # });
//! ```

pub use tower_guard_rs::{
    BLOCKED_MESSAGE, BoxError, DetectConfig, DetectVerdict, FAILURE_MESSAGE, GuardBody, GuardLayer,
    GuardService, OVERSIZE_MESSAGE, Threat, default_config,
};

/// Build the Guard layer for an axum [`Router`](axum::Router).
///
/// Apply it with [`Router::layer`](axum::Router::layer) (every registered
/// route) or [`Router::route_layer`](axum::Router::route_layer) (routes only,
/// skipping the fallback). Chain
/// [`GuardLayer::with_body_cap`] to change the body buffering cap.
///
/// # Example
///
/// ```
/// use axum::Router;
/// use axum_guard_rs::{DetectConfig, with_guard};
///
/// let config = DetectConfig {
///     max_content_length: 10_000,
///     max_full_scan_bytes: 262_144,
///     preserve_attack_patterns: true,
///     semantic_threshold: 0.7,
///     threat_score_threshold: 1.0,
///     binary_min_run_length: 16,
/// };
///
/// let app: Router = Router::new().layer(with_guard(config));
/// # let _ = app;
/// ```
#[must_use]
pub fn with_guard(config: DetectConfig) -> GuardLayer {
    GuardLayer::new(config)
}
