pub mod db;
mod error;
use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
    routing::any,
};
use db::{DbClient, ScoreBoard};
pub use error::{Error, Result,ClientError,ClientErrorKind};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


/// All the message types that can be sent over the web socket
/// connection
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="method",content="body")]
#[serde(rename_all = "camelCase",rename_all_fields="camelCase")]
pub enum ClientMessage {
    AddMember { name: String },
    DeleteMember { name: String },
    UpdateScore { name: String, score: u64 },
    CreateScoreBoard,
    GetScoreBoard{ id: Uuid },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="method",content="body")]
#[serde(rename_all = "camelCase",rename_all_fields="camelCase")]
pub enum ClientResponse {
    CreateScoreBoard{
        id: Uuid
    },
    GetScoreBoard{
        scoreboard: ScoreBoard
    },
}

async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(async |socket| {
        match handle_socket(socket, state).await {
            Ok(_) => {}
            Err(err) => {} // FIXME
        };
    })
}

async fn handle_socket(mut socket: WebSocket, mut state: AppState) -> crate::Result<()> {
    while let Some(msg) = socket.recv().await {
        if let Message::Text(text) = msg? {
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(message) => {   
                    let response = handle_message(message, &mut state).await?;
                    let message = serde_json::to_string(&response)?;

                    socket.send(Message::Text(message.into())).await?;
                }
                Err(err) => {
                    dbg!(err);
                }
            }
        }
    }

    Ok(())
}

pub async fn handle_message(
    message: ClientMessage,
    state: &mut AppState
) -> Result<ClientResponse>{
    let redis = state.client();

    match message {
        ClientMessage::CreateScoreBoard =>{
            let board = ScoreBoard::new();
            let id = board.id();

            redis.set_scoreboard(board).await?;
            let response = ClientResponse::CreateScoreBoard { id };
            
            Ok(response)
        },
        ClientMessage::GetScoreBoard { id } =>{
            match redis.get_scoreboard(&id).await? {
                Some(scoreboard) => {
                    let response = ClientResponse::GetScoreBoard { scoreboard };
                    Ok(response)
                }
                None => {
                    let error = ClientError::not_found("Scoreboard not found");
                    Err(error.into())
                }
            }

        }
        _ => {

            Err(Error::UnsupportedMethod)
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    client: DbClient,
}

impl AppState {
    pub async fn new() -> crate::Result<Self> {
        let client = DbClient::new().await?;

        Ok(Self { client })
    }

    /// Get a reference to the connection
    pub fn client(&mut self) -> &mut DbClient {
        &mut self.client
    }
}

pub async fn router() -> crate::Result<()> {
    let state = AppState::new().await?;
    let app = Router::new().route("/ws", any(handler)).with_state(state);

    let listener = tokio::net::TcpListener::bind("[::1]:5000").await.unwrap();

    println!("Listening on port 5000");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn set_user() -> crate::Result<()> {
        Ok(())
    }
}
