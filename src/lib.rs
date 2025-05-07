mod api;
mod error;
pub mod db;
pub mod auth;
pub mod board;
use std::env;
use axum::{
    extract::{
        ws::{Message, WebSocket}, State, WebSocketUpgrade
    }, 
    response::Response, 
    routing::{any, post}, 
    Router
};
use db::{DbClient, ScoreBoard};
pub use error::{Error, Result,ClientError,ClientErrorKind};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
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
    pool: PgPool
}

impl AppState {
    pub async fn new() -> crate::Result<Self> {
        let client = DbClient::new().await?;
        let database_url = env::var("DATABASE_URL").unwrap();
        let pool = PgPoolOptions::new()
            .max_connections(15)
            .connect(&database_url)
            .await?;

        Ok(Self { client,pool })
    }

    pub async fn with_pool(pool: PgPool) -> crate::Result<Self> {
        let client = DbClient::new().await?;

        Ok(Self { client,pool })
    }

    /// Get a reference to the client
    pub fn client(&mut self) -> &mut DbClient {
        &mut self.client
    }

    /// Get a reference to the database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

pub fn router(state: AppState) -> Router{
    let api = Router::new()
        .route("/auth/sign-up/anonymous", post(api::anon_sign_up));

    Router::new()
        .route("/ws", any(handler))
        .nest("/api/v1", api)
        .with_state(state)
}

pub async fn main() -> crate::Result<()> {
    let state = AppState::new().await?;
    let app = router(state);

    let listener = tokio::net::TcpListener::bind("[::1]:5000").await.unwrap();

    println!("Listening on port 5000");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
