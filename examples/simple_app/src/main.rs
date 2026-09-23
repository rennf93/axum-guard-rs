//! Minimal guarded axum application, using the real adapter surface:
//! [`axum_guard_rs::with_guard`] dropped into `Router::layer`.
//!
//! Routes:
//!
//! | Route | Guard | Behavior |
//! |---|---|---|
//! | `GET /health` | excluded | `200 ok` |
//! | `GET /` | guarded | `200` greeting |
//! | `GET /search?q=...` | guarded | `200 search ok`, or `403` when the query trips the engine |
//! | `POST /echo` | guarded | echoes the body; `403` for a threat, `413` over the body cap |
//!
//! The excluded path is expressed with axum's own scoping: the guarded routes
//! live in a sub-router that carries the `with_guard` layer, and
//! `Router::merge` combines it with an unguarded `/health` router. Middleware
//! added via `Router::layer` stays attached to the routes of the router it
//! was added to, which is exactly the Python distro's excluded-paths effect.

use axum::Router;
use axum::body::Bytes;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum_guard_rs::{default_config, with_guard};

#[tokio::main]
async fn main() {
    let addr: std::net::SocketAddr = std::env::var("APP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("APP_ADDR must be a socket address");

    let guarded = Router::new()
        .route("/", get(root))
        .route("/search", get(search))
        .route("/echo", post(echo))
        .layer(with_guard(default_config()));

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(guarded);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|error| panic!("failed to bind {addr}: {error}"));
    eprintln!("axum-guard-rs simple app listening on {addr}");
    axum::serve(listener, app).await.expect("server error");
}

async fn root() -> &'static str {
    "axum-guard-rs simple app"
}

async fn search() -> &'static str {
    "search ok"
}

async fn echo(body: Bytes) -> impl IntoResponse {
    (StatusCode::OK, String::from_utf8_lossy(&body).into_owned())
}
