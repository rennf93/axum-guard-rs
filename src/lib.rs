//! # axum-guard-rs
//!
//! Application-layer security middleware for
//! [axum](https://github.com/tokio-rs/axum), part of the
//! [Guard ecosystem](https://github.com/rennf93).
//!
//! ## Status: scaffold
//!
//! This crate is an intentionally minimal scaffold. The Guard Rust engine
//! ([guard-core-rs](https://github.com/rennf93/guard-core-rs)) is not yet a
//! published, consumable crate, so there is no integration code here yet.
//! What this scaffold establishes is package metadata, CI governance, and
//! the integration contract documented below, so the engine can be wired in
//! with minimal friction.
//!
//! Per the ecosystem boundary rules, adapter crates hold all framework glue
//! and no security logic: detection, rate limiting, and IP policy live in
//! the engine, never here.
//!
//! ## Planned integration: `tower::Layer` + `tower::Service`
//!
//! Axum middleware is tower middleware. The adapter will provide a
//! `GuardLayer` implementing `tower::Layer`, applied with `Router::layer`
//! or `Router::route_layer`, whose `layer` method wraps the inner service
//! in a `GuardService` implementing
//! `Service<axum::http::Request<axum::body::Body>>`. In `call`, that
//! service runs the Guard pipeline (IP reputation, rate limiting,
//! penetration-attempt detection, security headers) and either
//! short-circuits with a Guard-generated `axum::response::Response` or
//! forwards to the inner service, inspecting the response on the way out.
//!
//! The adapter will expose the layer roughly as follows (illustrative only;
//! the engine API does not exist yet):
//!
//! ```ignore
//! // Ignored on purpose: axum and tower are not dependencies of this
//! // scaffold, so this example cannot compile yet. It documents the shape
//! // the integration will take.
//! use axum::body::Body;
//! use axum::http::Request;
//! use axum::response::Response;
//! use tower::Layer;
//!
//! #[derive(Clone)]
//! pub struct GuardLayer {
//!     // engine configuration
//! }
//!
//! pub struct GuardService<S> {
//!     inner: S,
//! }
//!
//! impl<S> Layer<S> for GuardLayer {
//!     type Service = GuardService<S>;
//!
//!     fn layer(&self, inner: S) -> Self::Service {
//!         GuardService { inner }
//!     }
//! }
//! ```
//!
//! The `Service<Request<Body>>` implementation for `GuardService` (not
//! shown) is where request inspection and short-circuiting happen.
//!
//! ## Placeholder API
//!
//! [`add`] exists only so the scaffold has a testable public symbol while
//! the real API surface is designed. It will be removed when the engine
//! integration lands.

/// Placeholder smoke-test symbol for the scaffold.
///
/// It exists only so the crate has a testable public item while the real
/// API surface is designed; it will be removed when the engine integration
/// lands.
///
/// # Example
///
/// ```
/// assert_eq!(axum_guard_rs::add(2, 2), 4);
/// ```
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
