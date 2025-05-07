use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use scoreboard::{AppState, auth::User, router};
use sqlx::PgPool;
use tokio_tungstenite::connect_async;
use std::usize;
use tower::ServiceExt;

#[sqlx::test]
async fn connect_to_web_socket(pool: PgPool) -> scoreboard::Result<()> {
    let state = AppState::with_pool(pool).await?;
    let app = router(state.clone());

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/sign-up/anonymous")
        .header("Content-Type", "application/json")
        .body(Body::empty())?;

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let user: User = serde_json::from_slice(&bytes)?;
    connect_async("http://localhost:5000/ws").await?;

    let new_user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(state.pool())
        .await?;

    assert!(new_user.is_anonymous);

    Ok(())
}
