use std::collections::HashMap;
use crate::auth::gen_random_string;
use crate::db::ScoreBoard;
use crate::{auth, ClientError, Error, Result};
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{self, Sender};
use tokio::task::JoinHandle;
use crate::{AppState,ClientMessage,ClientResponse};

#[derive(Debug,Clone)]
pub struct ConnectionGroup{
    id: String,
    senders: Vec<mpsc::Sender<ClientMessage>>
}

impl ConnectionGroup{
    pub fn new() -> Self{
        let id = gen_random_string(12);
        Self { id, senders: vec![] }
    }

    pub fn add_connection(&mut self,tx: Sender<ClientMessage>){
        self.senders.push(tx);
    }

    pub async fn send_all(&mut self,message: ClientMessage) -> crate::Result<()>{
        for tx in &mut self.senders{
            // FIXME handle error
            let _ = tx.send(message.clone()).await;
        }

        Ok(())
    }
}

pub async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(async |socket| {
        match handle_socket(socket, state).await {
            Ok(_) => {}
            Err(err) => {} // FIXME
        };
    })
}

async fn handle_socket(mut socket: WebSocket, mut state: AppState) -> crate::Result<()> {
    let (mut sender,mut receiver) =  socket.split();
    let (tx,mut rx) = tokio::sync::mpsc::channel::<ClientMessage>(32);
    let mut group = ConnectionGroup::new();
    group.add_connection(tx);
    
    // Spawn a task to handle multiple messages
    let task: JoinHandle<Result<()>> = tokio::task::spawn(async move{
        while let Some(message) = rx.recv().await{
            let response = handle_message(message, &mut state).await?;
            let message = serde_json::to_string(&response)?;
            sender.send(Message::Text(message.into())).await?;
        }
        Ok(())
    });


    while let Some(msg) = receiver.next().await {
        if let Message::Text(text) = msg? {
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(message) => {
                    // tx.send(message).await;
                }
                Err(err) => {
                    dbg!(err);
                }
            }
        }
    }

    task.await;

    Ok(())
}

pub async fn handle_message(
    message: ClientMessage,
    state: &mut AppState,
) -> Result<ClientResponse> {
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
        _ => Err(Error::UnsupportedMethod),
    }
}


