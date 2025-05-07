use std::sync::Arc;
use crate::db::ScoreBoard;
use crate::AppState;
use crate::{ClientError, Error, Result};
use axum::extract::ws::Message;
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::WebSocket,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc,Mutex};
use tokio::task::JoinHandle;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone,Debug)]
pub struct ConnectionPool{
    senders: Arc<Mutex<Vec<mpsc::Sender<ClientMessage>>>>
}

impl ConnectionPool{
    pub fn new() -> Self{
        Self { 
            senders: Arc::new(Mutex::new(vec![])) 
        }
    }

    pub async fn add_connection(&mut self,tx:mpsc::Sender<ClientMessage>){
        self.senders
            .lock()
            .await
            .push(tx);
    }

    pub async fn send_all(&self,message: ClientMessage){
        for tx in self.senders.lock().await.iter(){
            let _ = tx.send(message.clone()).await;
        }
    }
}

/// All the message types that can be sent over the web socket
/// connection
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "body")]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ClientMessage {
    AddMember { name: String },
    DeleteMember { name: String },
    UpdateScore { name: String, score: u64 },
    CreateScoreBoard,
    JoinRoom{
        id: String,
    },
    GetScoreBoard { id: Uuid },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "body")]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ClientResponse {
    CreateScoreBoard { id: Uuid },
    GetScoreBoard { scoreboard: ScoreBoard },
    JoinRoom{
        id: String
    }
}

pub async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(async |socket| {
        match handle_socket(socket, state).await {
            Ok(_) => {}
            Err(_) => {} // FIXME
        };
    })
}

async fn handle_socket(socket: WebSocket, mut state: AppState) -> crate::Result<()> {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ClientMessage>(32);
    state.conn_pool().add_connection(tx);
    let mut duplicate = state.clone();


    // Spawn a task to handle multiple messages
    let task: JoinHandle<Result<()>> = tokio::task::spawn(async move {
        while let Some(message) = rx.recv().await {
            let response = handle_message(message, &mut duplicate).await?;
            let message = serde_json::to_string(&response)?;
            sender.send(Message::Text(message.into())).await?;
        }
        Ok(())
    });

    // Handle messages until the connection closes
    while let Some(msg) = receiver.next().await {
        if let Message::Text(text) = msg? {
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(message) => {
                    // FIXME handle error
                    state.conn_pool().send_all(message).await;
                }
                Err(err) => {
                    dbg!(err);
                }
            }
        }
    }

    // FIXME handle error
    // Wait for any remaining messages, the client might have already
    // stopped listening but it's still important
    let _ = task.await;

    Ok(())
}

pub async fn handle_message(
    message: ClientMessage,
    state: &mut AppState,
) -> Result<ClientResponse> {
    dbg!(state.conn_pool());
    let redis = state.client();
    match message {
        ClientMessage::CreateScoreBoard => {
            let board = ScoreBoard::new();
            let id = board.id();

            redis.set_scoreboard(board).await?;
            let response = ClientResponse::CreateScoreBoard { id };

            Ok(response)
        }
        ClientMessage::GetScoreBoard { id } => match redis.get_scoreboard(&id).await? {
            Some(scoreboard) => {
                let response = ClientResponse::GetScoreBoard { scoreboard };
                Ok(response)
            }
            None => {
                let error = ClientError::not_found("Scoreboard not found");
                Err(error.into())
            }
        },
        ClientMessage::JoinRoom { id } => {
            Ok(
                ClientResponse::JoinRoom { id: String::from("dlskdfd24@$") }
            )
        }
        _ => Err(Error::UnsupportedMethod),
    }
}
