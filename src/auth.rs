use std::time;
use base64::engine::{general_purpose::URL_SAFE,Engine};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug,Serialize,Deserialize,FromRow,Default)]
pub struct User{
    pub id: Uuid,
    pub email: Option<String>,
    pub user_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub phone_number: Option<String>,
    pub encrypted_password: Option<String>,
    pub is_anonymous: bool
}

/// Create an anonymous user in the database
pub async fn create_anon_user(pool: &PgPool) -> crate::Result<User>{
    let user = sqlx::query_as::<_,User>(
        "INSERT INTO users(is_anonymous) VALUES(true) RETURNING *",
    )
    .fetch_one(pool).await?;

    Ok(user)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[sqlx::test(migrations="./migrations")]
    async fn new_anon_user(pool: PgPool) -> crate::Result<()>{
        let user = create_anon_user(&pool).await?;
        assert_eq!(user.is_anonymous,true);

        Ok(())
    }
}