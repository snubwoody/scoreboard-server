pub mod api;
pub mod auth;
pub mod board;
pub mod db;
mod error;
pub mod ws;
use axum::{
    Router,
    routing::{any, get, post},
};
use db::{DbClient, ScoreBoard};
pub use error::{ClientError, ClientErrorKind, Error, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};
use ws::ConnectionPool;
use std::env;


#[derive(Clone)]
pub struct AppState {
    conn_pool: ConnectionPool,
    client: DbClient,
    pool: PgPool,
}

impl AppState {
    pub async fn new() -> crate::Result<Self> {
        let client = DbClient::new().await?;
        let database_url = env::var("DATABASE_URL").unwrap();
        let pool = PgPoolOptions::new()
            .max_connections(15)
            .connect(&database_url)
            .await?;

        Ok(Self {
            client,
            pool,
            conn_pool: ConnectionPool::new()
        })
    }

    pub async fn with_pool(pool: PgPool) -> crate::Result<Self> {
        let client = DbClient::new().await?;

        Ok(Self {
            client,
            pool,
            conn_pool: ConnectionPool::new()
        })
    }

    /// Get a reference to the client
    pub fn client(&mut self) -> &mut DbClient {
        &mut self.client
    }

    /// Get a reference to the database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get a reference to the connection pool
    pub fn conn_pool(&mut self) -> &mut ConnectionPool {
        &mut self.conn_pool
    }
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/auth/sign-up/anonymous", post(api::anon_sign_up))
        .route("/leaderboard", post(api::create_board))
        .route("/leaderboards", get(api::get_leaderboards));

    Router::new()
        .route("/ws", any(ws::handler))
        .nest("/api/v1", api)
        .with_state(state)
}

pub async fn main() -> crate::Result<()> {
    let _ = dotenv::dotenv();
    let state = AppState::new().await?;
    let app = router(state);

    let listener = tokio::net::TcpListener::bind("[::1]:5000").await.unwrap();

    println!("Listening on port 5000");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
