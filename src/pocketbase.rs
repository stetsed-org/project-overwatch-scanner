
use pocketbase_sdk::records::operations::create;
use pocketbase_sdk::user::UserTypes;
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
    pb_email: String,
    pb_password: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client =
        pocketbase_sdk::client::Client::new("https://pocketbase.selfhostable.net/api/").unwrap();
    let auth = client
        .auth_via_email(
            pb_email,
            pb_password,
            UserTypes::Admin, /* use UserTypes::Admin for admin Authentication */
        )
        .await;
    assert!(auth.is_ok());

    create::record::<Player>("global", &query, &client).await?;
    Ok(())
}
