
use pocketbase_sdk::{client, records::operations::create};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
    pub account: String,
    pub x: f64,
    pub z: f64,
    pub world: String,
}

pub async fn pocketbase_send(
    query: Player,
    client: &client::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    create::record::<Player>("global", &query, client).await?;
    Ok(())
}
