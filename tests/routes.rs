use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use scoreboard::{api, auth::User, board::Leaderboard, router, AppState};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::PgPool;
use std::usize;
use tower::ServiceExt;

struct RouteTest<B>{
    method: String,
    uri: String,
    body:Option<B>,
}

impl<B> RouteTest<B>
where B:Serialize
{
    fn new() -> Self{
        Self {  
            method: String::from("GET"),
            uri: String::from("/"),
            body: None,
        }
    }

    fn method(mut self,method: &str) -> Self{
        self.method = method.to_owned();
        self
    }

    fn uri(mut self,uri: &str) -> Self{
        self.uri = uri.to_owned();
        self
    }

    fn body(mut self,body: B) -> Self{
        self.body = Some(body);
        self
    }

    async fn send<R>(self,state: AppState) -> scoreboard::Result<(StatusCode,R)>
    where R: DeserializeOwned
    {
        let app = router(state);
        
        let mut body = Body::empty();
        
        if let Some(payload) = &self.body{
            let json = serde_json::to_string(&payload)?;
            body = Body::from(json);
        }
        
        let request = Request::builder()
            .method(self.method.as_str())
            .uri(self.uri)
            .header("Content-Type", "application/json")
            .body(body)?;
    
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();

        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
        let body: R = serde_json::from_slice(&bytes)?;

        Ok((status,body))
    }
}

#[sqlx::test]
async fn sign_in_anonymously(pool: PgPool) -> scoreboard::Result<()> {
    let state = AppState::with_pool(pool).await?;
    let app = router(state.clone());

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/sign-up/anonymous")
        .header("Content-Type", "application/json")
        .body(Body::empty())?;

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let user: User = serde_json::from_slice(&bytes)?;

    let new_user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(state.pool())
        .await?;

    assert!(new_user.is_anonymous);

    Ok(())
}


#[sqlx::test]
async fn create_a_leaderboard(pool: PgPool) -> scoreboard::Result<()> {
    let state = AppState::with_pool(pool).await?;
    
    let mut payload = api::CreateBoardPayload::default();
    payload.name = String::from("Leaderboard123");

    let (status,leaderboard) = RouteTest::new()
        .body(payload)
        .method("POST")
        .uri("/api/v1/leaderboard")
        .send::<Leaderboard>(state.clone())
        .await?;

    assert_eq!(status, StatusCode::CREATED);
    
    let new_board: Leaderboard = sqlx::query_as("SELECT * FROM leaderboards WHERE id = $1")
        .bind(leaderboard.id)
        .fetch_one(state.pool())
        .await?;

    assert_eq!(new_board.name,"Leaderboard123");

    Ok(())
}
