use serenity::{http::Http, model::id::ChannelId, model::Timestamp};

pub async fn send_message_to_channel(http: &Http, channel_id: ChannelId, content: String) {
    let msg = channel_id
        .send_message(&http, |m| {
            m.embed(|e| {
                e.title("Overwatch Notification")
                    .description(&content)
                    .footer(|f| f.text("Project Overwatch"))
                    .timestamp(Timestamp::now())
            })
        })
        .await;

    if let Err(why) = msg {
        println!("Error sending message: {:?}", why);
    }
}
