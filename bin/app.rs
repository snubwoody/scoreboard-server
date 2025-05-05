#[tokio::main]
async fn main() -> scoreboard::Result<()> {
    scoreboard::router().await?;

    Ok(())
}
