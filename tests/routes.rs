use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use scoreboard::{api, auth::User, board::Leaderboard, router, AppState};
use sqlx::PgPool;
use std::usize;
use tower::ServiceExt;

#[sqlx::test]
async fn sign_in_anonymously(pool: PgPool) -> scoreboard::Result<()> {
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

    let new_user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(state.pool())
        .await?;

    assert!(new_user.is_anonymous);

    Ok(())
}

#[sqlx::test]
async fn create_a_leaderboard(pool: PgPool) -> scoreboard::Result<()> {
    let state = AppState::with_pool(pool).await?;
    let app = router(state.clone());

    let mut payload = api::CreateBoardPayload::default();
    payload.name = String::from("Leaderboard123");
    
    let json = serde_json::to_string(&payload)?;
    let body = Body::from(json);
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/leaderboard")
        .header("Content-Type", "application/json")
        .body(body)?;

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let leaderboard: Leaderboard = serde_json::from_slice(&bytes)?;

    let new_board: Leaderboard = sqlx::query_as("SELECT * FROM leaderboards WHERE id = $1")
        .bind(leaderboard.id)
        .fetch_one(state.pool())
        .await?;

    assert_eq!(new_board.name,"Leaderboard123");

    Ok(())
}
