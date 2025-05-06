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

#[tokio::test]
async fn get_scoreboard() -> scoreboard::Result<()>{
    let mut state = AppState::new().await?;
    let message = ClientMessage::CreateScoreBoard;
    let response = handle_message(message, &mut state).await?;
    
    let id = match response {
        ClientResponse::CreateScoreBoard { id } => id,
        _ => panic!("Invalid response when creating scoreboard")
    };
    
    let message = ClientMessage::GetScoreBoard { id };
    let response = handle_message(message, &mut state).await?;

    match response {
        ClientResponse::GetScoreBoard { scoreboard } =>{
            assert_eq!(scoreboard.id(),id);
        },
        _ => panic!("Invalid response when creating scoreboard")
    }

    Ok(())
}

#[tokio::test]
async fn get_missing_scoreboard() -> scoreboard::Result<()>{
    let mut state = AppState::new().await?;
    let id = Uuid::new_v4();    
    let message = ClientMessage::GetScoreBoard { id };
    let response = handle_message(message, &mut state).await;
    
    match response.err().unwrap(){
        scoreboard::Error::ClientError(err) => {
            assert_eq!(err.kind(),ClientErrorKind::NotFound);
        },
        _ => panic!("Invalid error type")
    }    
    
    Ok(())
}