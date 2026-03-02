use axum::{routing::get, Json, Router};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub fn create_router(pool: PgPool) -> Router {
    let state = AppState { db: pool };

    Router::new()
        .route("/health", get(health_handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    /// Build a router backed by a real pool only when DATABASE_URL is set.
    /// Returns `None` when no DB is available so tests can be skipped cleanly.
    async fn try_build_router() -> Option<Router> {
        let url = std::env::var("DATABASE_URL").ok()?;
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .ok()?;
        Some(create_router(pool))
    }

    #[tokio::test]
    async fn health_endpoint_returns_200() {
        // The health endpoint doesn't touch the DB; use a lazy pool that never connects.
        let pool = PgPoolOptions::new()
            .max_connections(1)
            // connect_lazy skips the initial connection attempt
            .connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("lazy pool should be constructable");

        let app = create_router(pool);
        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok_json() {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("lazy pool should be constructable");

        let app = create_router(pool);
        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("lazy pool should be constructable");

        let app = create_router(pool);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[ignore = "requires a running PostgreSQL instance (set DATABASE_URL env var)"]
    async fn health_endpoint_with_real_db() {
        let app = try_build_router().await.expect("DATABASE_URL must be set");
        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
