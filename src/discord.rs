use serenity::{
    http::Http,
    model::id::ChannelId,
};

pub async fn send_message_to_channel(http: &Http, channel_id: ChannelId, content: String) {
    let result = channel_id.say(&http, content).await;

    match result {
        Err(why) => {
            println!("Error sending message: {:?}", why);
        }
        _ => (),
    }
}
