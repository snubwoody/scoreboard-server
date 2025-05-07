use scoreboard::{AppState, ws::{ClientMessage, ClientResponse, handle_message}};
use sqlx::PgPool;

#[sqlx::test]
async fn create_scoreboard(pool: PgPool) -> scoreboard::Result<()> {
    let mut state = AppState::with_pool(pool).await?;
    let message = ClientMessage::CreateScoreBoard;
    let response = handle_message(message, &mut state).await?;

    assert!(matches!(
        response,
        ClientResponse::CreateScoreBoard { id: _ }
    ));
    Ok(())
}
