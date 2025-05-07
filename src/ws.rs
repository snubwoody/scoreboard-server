use crate::db::ScoreBoard;
use crate::{AppState, ClientMessage, ClientResponse};
use crate::{ClientError, Error, Result};
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::WebSocket,
    },
    response::Response,
};

pub async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(async |socket| {
        match handle_socket(socket, state).await {
            Ok(_) => {}
            Err(_) => {} // FIXME
        };
    })
}

async fn handle_socket(_socket: WebSocket, _state: AppState) -> crate::Result<()> {
    // let (mut sender, mut receiver) = socket.split();
    // let (tx, mut rx) = tokio::sync::mpsc::channel::<ClientMessage>(32);
    // let mut group = ConnectionGroup::new();
    // group.add_connection(tx);

    // // Spawn a task to handle multiple messages
    // let task: JoinHandle<Result<()>> = tokio::task::spawn(async move {
    //     while let Some(message) = rx.recv().await {
    //         let response = handle_message(message, &mut state).await?;
    //         let message = serde_json::to_string(&response)?;
    //         sender.send(Message::Text(message.into())).await?;
    //     }
    //     Ok(())
    // });

    // while let Some(msg) = receiver.next().await {
    //     if let Message::Text(text) = msg? {
    //         match serde_json::from_str::<ClientMessage>(&text) {
    //             Ok(_) => {
    //                 // tx.send(message).await;
    //             }
    //             Err(err) => {
    //                 dbg!(err);
    //             }
    //         }
    //     }
    // }

    // FIXME handle error
    // let _ = task.await;

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
