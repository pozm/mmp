use sqlx::{pool::PoolConnection, Sqlite, SqliteConnection};
use tracing::instrument;

#[instrument]
pub async fn init_db(mut db: PoolConnection<Sqlite>) -> eyre::Result<()> {
    sqlx::query!(
        r#"CREATE TABLE IF NOT EXISTS users (
        user_id INTEGER PRIMARY KEY,
        username TEXT NOT NULL,
        password TEXT NOT NULL,
        email TEXT NOT NULL,
        role integer NOT NULL default 0
    )"#
    )
    .execute(&mut *db)
    .await?;
    sqlx::query!(
        r#"CREATE TABLE IF NOT EXISTS user_track_rating (
        user_id INTEGER NOT NULL,
        track_id TEXT NOT NULL
    )"#
    )
    .execute(&mut *db)
    .await?;
    Ok(())
}
