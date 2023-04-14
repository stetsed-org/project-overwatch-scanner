mod pocketbase;

use std::env;
use serde::{Deserialize, Serialize};
use std::fs;
use reqwest::*;
use sqlx::mysql::MySqlPool;
use anyhow::Result;
use dotenv::dotenv;
use pocketbase::pocketbase_send;

use serenity::{
    http::Http,
    model::id::ChannelId,
    prelude::*,
};

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
    let channel_id_str = env::var("DISCORD_CHANNEL_ID").expect("Expected a channel ID in the environment");
    let channel_id = ChannelId(channel_id_str.parse::<u64>().expect("Couldn't parse channel ID"));
    let pb_email: String = env::var("POCKETBASE_EMAIL").expect("Expected a pocketbase email in the environment");
    let pb_password: String = env::var("POCKETBASE_PASSWORD").expect("Expected a pocketbase password in the environment");


    let http = Http::new(&token);

    let pool: &MySqlPool = &MySqlPool::connect("mysql://data:data@truenas.selfhostable.net/data").await.expect("Failed to connect to database!");

    let contents: String = fs::read_to_string("./configuration.json")?;

    let config: Configuration = serde_json::from_str(&contents)?;

    let url: String = format!("{}up/world/{}/0", config.server, &config.world);

    let raw_json = get_request(url).await?;
    let player_info = extract_player_info(raw_json.clone());

    for player in &player_info {
        println!("Account: {}, X: {}, Z: {}", player.account, player.x, player.z);
        insert_entry(&pool, player.account.clone(), player.x as i64, player.z as i64).await?;
        let data = pocketbase::Player { account: player.account.clone(), x: player.x, z: player.z };
        pocketbase_send(data, pb_email.clone(), pb_password.clone()).await;
    }

    let in_our_land: Vec<String> = player_info
        .iter()
        .flat_map(|player| check_player_region(player, &config))
        .collect();

    println!("{:?}", in_our_land);

    for player in &player_info {
        if config.allylist.contains(&player.account) {
            continue;
        }
        else{
            if in_our_land.contains(&player.account) {
                let player_already_active = player_in_active(&pool, &player.account).await?;
                if player_already_active {
                    insert_active_entry(&pool, player.account.clone(), player.x as i64, player.z as i64).await?;
                }
                if !player_already_active {
                    print!("Somebody has entered our land hook.");
                    insert_active_entry(&pool, player.account.clone(), player.x as i64, player.z as i64).await?;
                    send_message_to_channel(&http, channel_id, format!("{} has entered our land! In {} <@&1084742210265813002>", player.account, player.world)).await;
                }
            }
            else {
                let player_already_active = player_in_active(&pool, &player.account).await?;
                if player_already_active {
                    print!("Somebody has left our land hook.");
                    print!("Put Image Logic Here to Download from database and render.");
                    sqlx::query!(r#"DELETE FROM active WHERE Name = ?"#, player.account).execute(pool).await.expect("Error deleting entry");
                    send_message_to_channel(&http, channel_id, format!("{} has left our land!", player.account)).await;
                }
        
            }
        }
    }

    pool.close().await;

    Ok(())
}
async fn get_request(url: String) -> Result<String> {
    let res = reqwest::get(url).await?;
    let body = res.text().await?;
    Ok(body)
}

fn extract_player_info(json_string: String) -> Vec<Player> {
    let parsed_json = serde_json::from_str::<serde_json::Value>(&json_string).unwrap();
    let players = parsed_json["players"].as_array().unwrap();

    let mut result = vec![];
    for player in players {
        let account = player["account"].as_str().unwrap().to_owned();
        let x = player["x"].as_f64().unwrap();
        let z = player["z"].as_f64().unwrap();
        let world = player["world"].as_str().unwrap().to_owned();
        result.push(Player { account, x, z, world});
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
    for (region_name, region) in &config.regions {
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

async fn insert_entry(pool: &MySqlPool, account: String, x: i64, z: i64) -> anyhow::Result<()> {
    sqlx::query!(r#"INSERT INTO global (Name,X,Z) VALUES (?,?,?)"#, account,x,z).execute(pool).await.expect("Error inserting entry");
    Ok(())
}

async fn insert_active_entry(pool: &MySqlPool, account: String, x: i64, z: i64) -> anyhow::Result<()> {
    sqlx::query!(r#"INSERT INTO active (Name,X,Z) VALUES (?,?,?)"#, account,x,z).execute(pool).await.expect("Error inserting entry");
    Ok(())
}

async fn player_in_active(pool: &MySqlPool, account: &str) -> Result<bool> {
    let row = sqlx::query!("SELECT Name from active WHERE Name = ?", account).fetch_optional(pool).await.expect("Failed to query active table.");
    Ok(row.is_some())
}

async fn send_message_to_channel(http: &Http, channel_id: ChannelId, content: String) {
    let result = channel_id.say(&http, content).await;

    match result {
        Err(why) => {
            println!("Error sending message: {:?}", why);
        }
        _ => (),
    }
}
