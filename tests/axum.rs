//! axum-specific behavior: `Router::layer` + `with_guard`, exercised end to
//! end through `tower::ServiceExt::oneshot` with `axum::body::Body`.

use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_guard_rs::{BLOCKED_MESSAGE, OVERSIZE_MESSAGE, default_config, with_guard};
use http::StatusCode;
use http::header::CONTENT_TYPE;
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::{Service, ServiceExt};

/// A small router with a variety of handler shapes.
fn app() -> Router {
    Router::new()
        .route("/hello", get(|| async { "hello" }))
        .route("/state", get(read_state))
        .route("/echo", post(echo_json))
        .route("/files/{*path}", get(|| async { "files" }))
        .layer(with_guard(default_config()))
        .with_state(Arc::new("state reached".to_owned()))
}

async fn read_state(State(state): State<Arc<String>>) -> String {
    (*state).clone()
}

async fn echo_json(Json(value): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(value)
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

fn post_request(uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_owned()))
        .expect("request")
}

async fn body_text(response: axum::response::Response) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[tokio::test]
async fn benign_get_passes_through_the_router() {
    let response = app()
        .oneshot(get_request("/hello"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_text(response).await, "hello");
}

#[tokio::test]
async fn json_body_round_trips_through_extraction_and_guard() {
    let response = app()
        .oneshot(post_request("/echo", r#"{"name":"renn","id":7}"#))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    // Compare as parsed values: `serde_json::Value` re-serializes map keys in
    // sorted order, so byte-equality against the request body would not hold.
    let text = body_text(response).await;
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("json");
    assert_eq!(parsed, serde_json::json!({"name": "renn", "id": 7}));
}

#[tokio::test]
async fn handler_state_extraction_is_unaffected() {
    let response = app()
        .oneshot(get_request("/state"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_text(response).await, "state reached");
}

#[tokio::test]
async fn xss_payload_in_body_is_blocked() {
    let response = app()
        .oneshot(post_request("/echo", r"<script>alert(1)</script>"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).expect("content type"),
        "application/json"
    );
    assert_eq!(
        body_text(response).await,
        format!(r#"{{"detail":"{BLOCKED_MESSAGE}"}}"#)
    );
}

#[tokio::test]
async fn traversal_payload_in_path_is_blocked_in_url_path_context() {
    let response = app()
        .oneshot(get_request("/files/../../etc/passwd"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn command_injection_in_query_is_blocked_in_query_param_context() {
    let response = app()
        .oneshot(get_request("/hello?cmd=$(whoami)"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn xss_payload_in_scanned_header_is_blocked() {
    let request = Request::builder()
        .uri("/hello")
        .header("x-comment", "<script>alert(1)</script>")
        .body(Body::empty())
        .expect("request");
    let response = app().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn benign_headers_do_not_trip_the_guard() {
    let request = Request::builder()
        .uri("/hello")
        .header(
            "authorization",
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.sig",
        )
        .header("cookie", "session=5f4dcc3b5aa765d61d8327deb882cf99")
        .header("user-agent", "guard-tests/0.1")
        .body(Body::empty())
        .expect("request");
    let response = app().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn body_over_the_cap_is_rejected_with_413() {
    let app = Router::new()
        .route("/echo", post(echo_json))
        .layer(with_guard(default_config()).with_body_cap(16));
    let response = app
        .oneshot(post_request(
            "/echo",
            r#"{"note":"this is longer than sixteen bytes"}"#,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        body_text(response).await,
        format!(r#"{{"detail":"{OVERSIZE_MESSAGE}"}}"#)
    );
}

#[tokio::test]
async fn unmatched_route_still_answers_404() {
    // The guard layer wraps registered routes; a request that matches nothing
    // reaches the fallback untouched. This pins that the guard does not turn
    // unrelated 404s into blocks.
    let response = app().oneshot(get_request("/nope")).await.expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn concurrent_requests_are_screened_independently() {
    let app = app();
    let handles: Vec<_> = (0..16)
        .map(|index| {
            let app = app.clone();
            tokio::spawn(async move {
                let request = if index % 2 == 0 {
                    get_request("/hello")
                } else {
                    post_request("/echo", r"<script>alert(1)</script>")
                };
                app.oneshot(request).await.expect("response").status()
            })
        })
        .collect();

    for (index, handle) in handles.into_iter().enumerate() {
        let status = handle.await.expect("task");
        if index % 2 == 0 {
            assert_eq!(status, StatusCode::OK, "benign request {index}");
        } else {
            assert_eq!(status, StatusCode::FORBIDDEN, "threat request {index}");
        }
    }
}

#[tokio::test]
async fn poll_ready_forwards_through_the_router() {
    let mut app = app();
    let waker = std::task::Waker::noop();
    let mut cx = std::task::Context::from_waker(waker);
    match Service::<Request<Body>>::poll_ready(&mut app, &mut cx) {
        std::task::Poll::Ready(result) => result.expect("ready"),
        std::task::Poll::Pending => panic!("router is always ready"),
    }
}
