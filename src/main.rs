mod discord;
mod pocketbase;
mod sql;

use anyhow::Result;
use dotenv::dotenv;
use pocketbase_sdk::user::UserTypes;
use serde::{Deserialize, Serialize};
use serenity::http::Http;
use serenity::model::id::ChannelId;
use std::env;
use std::fs;

use rusqlite::Connection;

use discord::*;
use pocketbase::*;
use sql::*;

#[derive(Debug, Deserialize, Serialize)]
struct Configuration {
    channel: String,
    server: String,
    world: String,
    allylist: Vec<String>,
    regions: std::collections::HashMap<String, Region>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Region {
    a: [i32; 2],
    b: [i32; 2],
}

#[derive(Debug, Deserialize, Serialize)]
struct Player {
    account: String,
    x: f64,
    z: f64,
    world: String,
    region: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    loop {
        main_function().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}

async fn main_function() -> anyhow::Result<()> {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let channel_id_str =
        env::var("DISCORD_CHANNEL_ID").expect("Expected a channel ID in the environment");
    let channel_id = ChannelId(
        channel_id_str
            .parse::<u64>()
            .expect("Couldn't parse channel ID"),
    );
    let pb_email: String =
        env::var("POCKETBASE_EMAIL").expect("Expected a pocketbase email in the environment");
    let pb_password: String =
        env::var("POCKETBASE_PASSWORD").expect("Expected a pocketbase password in the environment");
    let pb_api_route: String = env::var("POCKETBASE_API_ROUTE")
        .expect("Expected a pocketbase api route in the environment");

    let http = Http::new(&token);

    let conn = Connection::open("main.db")?;

    let mut client = pocketbase_sdk::client::Client::new(&pb_api_route).unwrap();
    let _auth = client
        .auth_via_email(
            pb_email,
            pb_password,
            UserTypes::Admin, /* use UserTypes::Admin for admin Authentication */
        )
        .await;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS active (
        ID INTEGER PRIMARY KEY AUTOINCREMENT,
        Name TEXT,
        Date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        X INTEGER,
        Z INTEGER
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS global (
        ID INTEGER PRIMARY KEY AUTOINCREMENT,
        Name TEXT,
        Date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        X INTEGER,
        Z INTEGER
        )",
        (),
    )?;

    let contents: String = fs::read_to_string("./configuration.json")?;

    let config: Configuration = serde_json::from_str(&contents)?;

    let url: String = format!("{}up/world/{}/0", config.server, &config.world);

    let raw_json = get_request(url).await?;
    let player_info = extract_player_info(raw_json.clone(), &config);

    for player in &player_info {
        println!(
            "Account: {}, X: {}, Z: {}",
            player.account, player.x, player.z
        );
        insert_entry(
            &conn,
            player.account.clone(),
            player.x as i64,
            player.z as i64,
        )
        .await?;
        let data = pocketbase::Player {
            account: player.account.clone(),
            x: player.x,
            z: player.z,
            world: player.world.clone(),
        };
        pocketbase_send(data, &client)
            .await
            .expect("Error sending to pocketbase database");
    }

    let in_our_land: Vec<String> = player_info
        .iter()
        .flat_map(|player| check_player_region(player, &config))
        .collect();

    println!("{:?}", in_our_land);

    for player in &player_info {
        if config.allylist.contains(&player.account) {
            continue;
        } else {
            if in_our_land.contains(&player.account) {
                let player_already_active = player_in_active(&conn, &player.account).await?;
                if player_already_active {
                    insert_active_entry(
                        &conn,
                        player.account.clone(),
                        player.x as i64,
                        player.z as i64,
                    )
                    .await?;
                }
                if !player_already_active {
                    print!("Somebody has entered our land hook.");
                    insert_active_entry(
                        &conn,
                        player.account.clone(),
                        player.x as i64,
                        player.z as i64,
                    )
                    .await?;
                    send_message_to_channel(
                        &http,
                        channel_id,
                        format!(
                            "{} has entered our land, at the location {}! In {} <@&1084742210265813002>",
                            player.account, player.world, player.region
                        ),
                    )
                    .await;
                }
            } else {
                let player_already_active = player_in_active(&conn, &player.account).await?;
                if player_already_active {
                    print!("Somebody has left our land hook.");
                    print!("Put Image Logic Here to Download from database and render.");
                    delete_in_active(&conn, &player.account).await?;
                    send_message_to_channel(
                        &http,
                        channel_id,
                        format!("{} has left our land!", player.account),
                    )
                    .await;
                }
            }
        }
    }
    if let Err(_e) = conn.close() {
        println!("Failed to close database");
    }

    Ok(())
}
async fn get_request(url: String) -> Result<String> {
    let res = reqwest::get(url).await?;
    let body = res.text().await?;
    Ok(body)
}

fn extract_player_info(json_string: String, config: &Configuration) -> Vec<Player> {
    let parsed_json = serde_json::from_str::<serde_json::Value>(&json_string).unwrap();
    let players = parsed_json["players"].as_array().unwrap();

    let mut result = vec![];
    for player in players {
        let account = player["account"].as_str().unwrap().to_owned();
        let x = player["x"].as_f64().unwrap();
        let z = player["z"].as_f64().unwrap();
        let world = player["world"].as_str().unwrap().to_owned();
        let check_player_region_name = check_player_region_name(
            &Player {
                account: account.clone(),
                x,
                z,
                world: world.clone(),
                region: "".to_string(),
            },
            config,
        );
        let region = if check_player_region_name.len() > 0 {
            check_player_region_name[0].clone()
        } else {
            "".to_string()
        };
        result.push(Player {
            account,
            x,
            z,
            world,
            region,
        });
    }

    result
}

fn check_player_region(player: &Player, config: &Configuration) -> Vec<String> {
    let mut in_our_land = Vec::new();
    let (mut a_divisor, mut b_divisor) = (1.0, 1.0);
    if player.world == "world_nether" {
        a_divisor = 8.0;
        b_divisor = 8.0;
    }
    for (_region_name, region) in &config.regions {
        if player.x >= region.a[0] as f64 / a_divisor
            && player.x <= region.b[0] as f64 / b_divisor
            && player.z >= region.a[1] as f64 / a_divisor
            && player.z <= region.b[1] as f64 / b_divisor
        {
            in_our_land.push(player.account.clone());
            break; // Assuming player can only be in one region at a time
        }
    }
    in_our_land
}

fn check_player_region_name(player: &Player, config: &Configuration) -> Vec<String> {
    let mut in_our_land = Vec::new();
    let (mut a_divisor, mut b_divisor) = (1.0, 1.0);
    if player.world == "world_nether" {
        a_divisor = 8.0;
        b_divisor = 8.0;
    }
    for (region_name, region) in &config.regions {
        if player.x >= region.a[0] as f64 / a_divisor
            && player.x <= region.b[0] as f64 / b_divisor
            && player.z >= region.a[1] as f64 / a_divisor
            && player.z <= region.b[1] as f64 / b_divisor
        {
            in_our_land.push(region_name.clone());
            break; // Assuming player can only be in one region at a time
        }
    }
    in_our_land
}
