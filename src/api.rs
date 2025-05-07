use axum::{extract::State, http::StatusCode, Json};
use crate::{auth::{self, User}, AppState};



/// Sign up as an anonymous user
pub async fn anon_sign_up(
    State(state): State<AppState>
) -> crate::Result<(StatusCode,Json<User>)>
{
    let user = auth::create_anon_user(&state.pool).await?;
    let response = (StatusCode::CREATED,Json(user));

    Ok(response)
}

#[cfg(test)]
mod tests{
    use sqlx::PgPool;
    use super::*;

    #[sqlx::test(migrations="./migrations")]
    async fn sign_up_anonymously(pool: PgPool) -> crate::Result<()>{
        let state = AppState::with_pool(pool).await?;
        let (status,_) = anon_sign_up(State(state)).await?;
        
        assert_eq!(status,StatusCode::CREATED);
        Ok(())
    }
}