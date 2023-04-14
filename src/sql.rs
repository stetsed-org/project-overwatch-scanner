use sqlx::mysql::MySqlPool;
use anyhow::Result;

pub async fn insert_entry(pool: &MySqlPool, account: String, x: i64, z: i64) -> anyhow::Result<()> {
    sqlx::query!(r#"INSERT INTO global (Name,X,Z) VALUES (?,?,?)"#, account,x,z).execute(pool).await.expect("Error inserting entry");
    Ok(())
}

pub async fn insert_active_entry(pool: &MySqlPool, account: String, x: i64, z: i64) -> anyhow::Result<()> {
    sqlx::query!(r#"INSERT INTO active (Name,X,Z) VALUES (?,?,?)"#, account,x,z).execute(pool).await.expect("Error inserting entry");
    Ok(())
}

pub async fn player_in_active(pool: &MySqlPool, account: &str) -> Result<bool> {
    let row = sqlx::query!("SELECT Name from active WHERE Name = ?", account).fetch_optional(pool).await.expect("Failed to query active table.");
    Ok(row.is_some())
}
