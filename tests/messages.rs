use scoreboard::{handle_message, AppState, ClientErrorKind, ClientMessage, ClientResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn create_scoreboard(pool: PgPool) -> scoreboard::Result<()>{
    let mut state = AppState::new().await?;
    let message = ClientMessage::CreateScoreBoard;
    let response = handle_message(message, &mut state).await?;

    assert!(matches!(response,ClientResponse::CreateScoreBoard{ id: _} ));
    Ok(())
}