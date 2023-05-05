use anyhow::Result;
use rusqlite::Connection;

pub async fn insert_entry(
    conn: &Connection,
    account: String,
    x: i64,
    z: i64,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO global (Name,X,Z) VALUES (?1, ?2, ?3)",
        [&account, &x.to_string(), &z.to_string()],
    )
    .expect("Error inserting entry");
    Ok(())
}

pub async fn insert_active_entry(
    conn: &Connection,
    account: String,
    x: i64,
    z: i64,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO active (Name,X,Z) VALUES (?1, ?2, ?3)",
        [&account, &x.to_string(), &z.to_string()],
    )
    .expect("Error inserting entry");
    Ok(())
}

pub async fn delete_in_active(conn: &Connection, account: &str) -> anyhow::Result<()> {
    conn.execute("DELETE FROM active WHERE Name = ?1", [&account])
        .expect("Error inserting entry");
    Ok(())
}

pub async fn player_in_active(conn: &Connection, account: &str) -> Result<bool> {
    let row = conn
        .execute("DELETE FROM active WHERE Name = ?1", [&account])
        .expect("Error inserting entry");
    Ok(row > 0)
}
