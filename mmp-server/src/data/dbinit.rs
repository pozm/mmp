use sqlx::SqliteConnection;
use tracing::instrument;

#[instrument]
async fn init_db(db: &mut SqliteConnection) -> eyre::Result<()> {
    sqlx::query!(
        r#"CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        username TEXT NOT NULL,
        password TEXT NOT NULL,
        email TEXT NOT NULL
    )"#
    )
    .execute(db)
    .await?;
    Ok(())
}
