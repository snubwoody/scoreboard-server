use crate::{
    auth::{self, User}, board::{self, Leaderboard}, AppState
};
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize,Deserialize,Default)]
pub struct CreateBoardPayload{
    pub name: String
}

/// Sign up as an anonymous user
pub async fn anon_sign_up(
    State(state): State<AppState>,
) -> crate::Result<(StatusCode, Json<User>)> {
    let user = auth::create_anon_user(&state.pool).await?;
    let response = (StatusCode::CREATED, Json(user));

    Ok(response)
}

/// Create a leaderboard
pub async fn create_board(
    State(state): State<AppState>,
    Json(payload): Json<CreateBoardPayload>,
) -> crate::Result<(StatusCode,Json<Leaderboard>)> {
    let board = board::Leaderboard::new(&payload.name, state.pool()).await?;

    Ok((StatusCode::CREATED,Json(board)))
}

/// Get all leaderboards
pub async fn get_leaderboards(
    State(state): State<AppState>,
) -> crate::Result<Json<Vec<Leaderboard>>> {
    let boards: Vec<Leaderboard> = sqlx::query_as("SELECT * FROM leaderboards")
        .fetch_all(state.pool())
        .await?;

    Ok(Json(boards))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(migrations = "./migrations")]
    async fn sign_up_anonymously(pool: PgPool) -> crate::Result<()> {
        let state = AppState::with_pool(pool).await?;
        let (status, _) = anon_sign_up(State(state)).await?;

        assert_eq!(status, StatusCode::CREATED);
        Ok(())
    }
}
