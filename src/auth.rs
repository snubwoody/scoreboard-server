use std::time;
use base64::engine::{general_purpose::URL_SAFE,Engine};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug,Serialize,Deserialize,FromRow)]
pub struct User{
    pub id: Uuid,
    pub email: Option<String>,
    pub user_name: Option<String>
}

#[derive(Debug,Serialize,Deserialize,FromRow,PartialEq, Eq,Clone)]
pub struct RefreshToken{
    pub id: Uuid,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub session_id: Uuid,
    pub active: bool,
}

#[derive(Debug,Serialize,Deserialize,FromRow,PartialEq, Eq)]
pub struct SessionToken{
    pub id: Uuid,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub user_id: Uuid,
}

pub async fn check_user_auth(){

}

/// Create a 32 byte string filled with
/// random data.
pub fn create_token() -> String{
    let bytes:[u8;32] = rand::random();
    URL_SAFE.encode(bytes)
}


/// Create a session token and insert it 
/// into the database.
pub async fn create_session_token(
    user_id: Uuid,
    pool: &PgPool
) -> crate::Result<SessionToken>{
    let session_token = create_token();
    let expiry = Utc::now() + time::Duration::from_secs(60 * 30);
    
    let token = 
        sqlx::query_as::<_,SessionToken>(
            "INSERT INTO session_tokens(token,expires_at,user_id) VALUES($1,$2,$3) RETURNING *"
        )
        .bind(session_token)
        .bind(expiry)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(token)
}

pub async fn create_refresh_token(session_id: Uuid,pool: &PgPool) -> crate::Result<RefreshToken>{
    let token = create_token();
    
    let refresh_token = 
        sqlx::query_as::<_,RefreshToken>(
            "INSERT INTO refresh_tokens(token,session_id,active) 
            VALUES($1,$2,true) 
            RETURNING *
            "
        )
        .bind(token)
        .bind(session_id)
        .fetch_one(pool)
        .await?;

    Ok(refresh_token)

}

pub async fn create_token_pair(){

}

/// Refresh the user tokens
pub async fn refresh_tokens(){

}

/// Create a user in the database
pub async fn create_user(pool: &PgPool,desc: UserDesc) -> crate::Result<User>{
    let user = sqlx::query_as::<_,User>(
        "INSERT INTO users(id) VALUES($1) RETURNING *",
    )
    .bind(desc.id)
    .fetch_one(pool).await?;

    Ok(user)
}

#[derive(Debug,Clone,Default)]
pub struct UserDesc{
    id: Uuid,
    email: Option<String>,
    user_name: Option<String>
}

impl UserDesc{
    pub fn new() -> Self{
        Self { 
            id: Uuid::new_v4(), 
            email: None, 
            user_name: None 
        }
    }

    pub fn email(&mut self,email: &str){
        self.email = Some(email.to_owned());
    }

    pub fn user_name(&mut self,user_name: &str){
        self.user_name = Some(user_name.to_owned());
    } 
}



#[cfg(test)]
mod tests{
    use super::*;

    #[sqlx::test(migrations="./migrations")]
    async fn new_session_token(pool: PgPool) -> crate::Result<()>{
        let user = create_user(&pool,Default::default()).await?;
        let token = create_session_token(user.id,&pool).await?;
        
        let session_token = sqlx::query_as::<_,SessionToken>(
            "SELECT * FROM session_tokens WHERE id = $1"
        )
        .bind(token.id)
        .fetch_one(&pool)
        .await?;

        let max_expiry = Utc::now() + time::Duration::from_secs(60 * 30);

        assert_eq!(token,session_token);
        assert!(session_token.expires_at <= max_expiry);
        Ok(())
    }

    #[sqlx::test(migrations="./migrations")]
    async fn new_refresh_token(pool: PgPool) -> crate::Result<()>{
        let user = create_user(&pool,Default::default()).await?;
        let session_token = create_session_token(user.id,&pool).await?;
        let refresh_token = create_refresh_token(session_token.id,&pool).await?;
        
        let token = sqlx::query_as::<_,RefreshToken>(
            "SELECT * FROM refresh_tokens WHERE id = $1"
        )
        .bind(refresh_token.id)
        .fetch_one(&pool)
        .await?;

        assert_eq!(token,refresh_token);
        Ok(())
    }

    #[sqlx::test]
    async fn create_new_user(pool: PgPool) -> crate::Result<()>{
        let mut user = UserDesc::new();
        user.email("wakunguma13@gmail.com");
        create_user(&pool, user).await?;

        let result = sqlx::query_as::<_,User>("SELECT * FROM users")
            .fetch_all(&pool)
            .await?;

        dbg!(result);

        Ok(())
    }
}