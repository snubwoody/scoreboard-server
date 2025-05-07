use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use uuid::Uuid;


#[derive(Debug,FromRow,Serialize,Deserialize)]
pub struct Leaderboard{
    pub id: i32,
    pub name: String
}

#[derive(Debug,FromRow,Serialize,Deserialize)]
pub struct LeaderboardMember{
    pub id: i32,
    pub leaderboard: i32,
    pub player_alias: Option<String>,
    pub player: Uuid
}

impl Leaderboard{
    /// Create a new [`Leaderboard`]
    /// 
    /// # Example
    /// ```
    /// use scoreboard::{board::Leaderboard,Error,AppState};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(),Error>{
    ///     dotenv::dotenv();
    /// 
    ///     let state = AppState::new().await?;
    ///     let board = Leaderboard::new("My leaderboard",state.pool())
    ///         .await?;
    /// 
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(name:&str,pool: &PgPool) -> crate::Result<Self>{
        let leaderboard: Leaderboard = 
            sqlx::query_as("INSERT INTO leaderboards(name) VALUES($1) RETURNING *")
            .bind(name)
            .fetch_one(pool)
            .await?;
    
        Ok(leaderboard)
    }

    /// Add a player to the board members
    pub async fn add_member(&self,player_id: Uuid,pool: &PgPool) -> crate::Result<()>{
        sqlx::query(
            "INSERT INTO leaderboard_members(player,leaderboard) 
            VALUES($1,$2) 
            RETURNING *"
        )
            .bind(player_id)
            .bind(self.id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Add a player to the board members
    pub async fn get_members(&self,pool: &PgPool) -> crate::Result<Vec<LeaderboardMember>>{
        let members: Vec<LeaderboardMember> = 
            sqlx::query_as("SELECT * FROM leaderboard_members WHERE leaderboard = $1")
            .bind(self.id)
            .fetch_all(pool)
            .await?;

        Ok(members)
    }
}

#[cfg(test)]
mod tests{
    use crate::auth::create_anon_user;
    use super::*;

    #[sqlx::test(migrations="./migrations")]
    async fn add_player_to_board(pool: PgPool) -> crate::Result<()>{
        let user = create_anon_user(&pool).await?;
        let board = Leaderboard::new("My leaderboard", &pool).await?;
        board.add_member(user.id, &pool).await?;
        
        let member: LeaderboardMember = 
            sqlx::query_as("SELECT * FROM leaderboard_members WHERE player = $1")
            .bind(user.id)
            .fetch_one(&pool)
            .await?;

        assert_eq!(member.player,user.id);

        Ok(())
    }

    #[sqlx::test(migrations="./migrations")]
    async fn get_board_members(pool: PgPool) -> crate::Result<()>{
        let user = create_anon_user(&pool).await?;
        let user2 = create_anon_user(&pool).await?;
        let user3 = create_anon_user(&pool).await?;
        
        let board = Leaderboard::new("My leaderboard", &pool).await?;
        board.add_member(user.id, &pool).await?;
        board.add_member(user2.id, &pool).await?;
        board.add_member(user3.id, &pool).await?;
        
        let members = board.get_members(&pool).await?;

        assert_eq!(members.len(),3);

        Ok(())
    }
}