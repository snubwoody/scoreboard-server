use redis::{AsyncCommands, aio::MultiplexedConnection};
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, FromRedisValue, ToRedisArgs)]
pub struct ScoreBoard {
    id: Uuid,
    users: Vec<User>,
}

impl ScoreBoard {
    pub fn new() -> Self {
        let id = Uuid::new_v4();

        Self { id, users: vec![] }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, Copy, FromRedisValue, ToRedisArgs)]
pub struct Score {
    value: u64, // TODO maybe make this f64
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, FromRedisValue, ToRedisArgs)]
pub struct User {
    id: Uuid,
    scores: Vec<Score>,
}

impl User {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_id(id: Uuid) -> Self {
        Self { id, scores: vec![] }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn add_score(&mut self, score: u64) {
        let score = Score { value: score };

        self.scores.push(score);
    }

    pub fn scores(&self) -> Vec<Score> {
        self.scores.clone()
    }

    /// Get the user's total score
    ///
    /// ```
    /// use scoreboard::User;
    ///
    /// let mut user = User::new();
    /// user.add_score(20);
    /// user.add_score(200);
    ///
    /// let total = user.total_score();
    ///
    /// assert_eq!(total,220);
    /// ```
    ///
    pub fn total_score(&self) -> u64 {
        self.scores.iter().fold(0, |acc, score| score.value + acc)
    }
}

#[derive(Clone)]
pub struct DbClient {
    connection: MultiplexedConnection,
}

impl DbClient {
    pub async fn new() -> crate::Result<Self> {
        let client = redis::Client::open("redis://[::1]:6379")?;
        let connection = client.get_multiplexed_async_connection().await?;

        Ok(Self { connection })
    }
    // TODO make generic get and set methods

    pub async fn set_scoreboard(&mut self, scoreboard: ScoreBoard) -> crate::Result<()> {
        let _: () = self
            .connection
            .set(format!("scoreboard:{}", scoreboard.id), scoreboard)
            .await?;

        Ok(())
    }

    pub async fn get_scoreboard(&mut self, id: &Uuid) -> crate::Result<Option<ScoreBoard>> {
        let response: Result<ScoreBoard, redis::RedisError> =
            self.connection.get(format!("scoreboard:{}", id)).await;

        match response {
            Ok(board) => Ok(Some(board)),
            Err(error) => match error.kind() {
                redis::ErrorKind::TypeError => Ok(None),
                _ => Err(error.into()),
            },
        }
    }

    pub async fn set_user(&mut self, user: User) -> crate::Result<()> {
        let _: () = self
            .connection
            .set(format!("user:{}", user.id), user)
            .await?;

        Ok(())
    }

    pub async fn get_user(&mut self, id: &Uuid) -> crate::Result<Option<User>> {
        let response: Result<User, redis::RedisError> =
            self.connection.get(format!("user:{}", id)).await;

        match response {
            Ok(user) => Ok(Some(user)),
            Err(error) => match error.kind() {
                redis::ErrorKind::TypeError => Ok(None),
                _ => Err(error.into()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_set_user() -> crate::Result<()> {
        let mut client = DbClient::new().await?;
        let id = Uuid::new_v4();
        client.set_user(User::from_id(id)).await?;

        let user = client.get_user(&id).await?.unwrap();
        assert_eq!(user.id, id);

        Ok(())
    }

    #[tokio::test]
    async fn get_set_scoreboard() -> crate::Result<()> {
        let mut client = DbClient::new().await?;
        let scoreboard = ScoreBoard::new();
        let id = scoreboard.id;
        client.set_scoreboard(scoreboard).await?;

        let scoreboard = client.get_scoreboard(&id).await?.unwrap();
        assert_eq!(scoreboard.id, id);

        Ok(())
    }

    #[tokio::test]
    async fn update_user_score() -> crate::Result<()> {
        let mut client = DbClient::new().await?;
        let user = User::new();
        let id = user.id;
        client.set_user(user).await?;

        let mut user = client.get_user(&id).await?.unwrap();
        assert_eq!(user.total_score(), 0);
        user.add_score(200);
        user.add_score(2);
        client.set_user(user).await?;

        let user = client.get_user(&id).await?.unwrap();
        assert_eq!(user.total_score(), 202);

        Ok(())
    }

    #[tokio::test]
    async fn missing_user_returns_none() -> crate::Result<()> {
        let mut client = DbClient::new().await?;
        let user = client.get_user(&Uuid::new_v4()).await?;
        assert!(user.is_none());

        Ok(())
    }
}
