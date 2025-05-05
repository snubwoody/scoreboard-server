pub mod db;
mod error;
mod state;
use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
    routing::any,
};
use db::{DbClient, User};
pub use error::{Error, Result};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ClientMessage {
    AddMember { name: String },
    DeleteMember { name: String },
    UpdateScore { name: String, score: u64 },
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientResponse {}

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
        match msg.unwrap() {
            Message::Text(text) => match serde_json::from_str::<ClientMessage>(&text) {
                Ok(message) => {
                    let user = User::new();
                    let id = user.id();
                    state.client().set_user(user).await?;

                    let user = state.client().get_user(&id).await?;
                    let msg = serde_json::to_string(&user)?;
                    socket.send(Message::text(msg)).await.unwrap();
                }
                Err(_) => {
                    dbg!("Invalid message");
                }
            },
            Message::Close(_) => {}
            _ => {}
        }
    }

    Ok(())
}

#[derive(Clone)]
struct AppState {
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
    let client = redis::Client::open("redis://[::1]:6379").unwrap();

    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let _: () = conn.set("user:5000", "hello").await.unwrap();

    let response: String = conn.get("user:5000").await.unwrap();
    dbg!(response);

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
