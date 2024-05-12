use serenity::all::{ChannelId, Http};

const CHARACTERS_LIMIT: usize = 2000;

pub(crate) struct Discord {
    http: Http,
    channel_id: ChannelId,
}

impl Discord {
    pub(crate) fn new(token: &str, id: u64) -> Discord {
        Discord {
            http: Http::new(token),
            channel_id: ChannelId::new(id),
        }
    }

    pub(crate) async fn send_discord_message(&self, message: &str) {
        let message = message.chars().take(CHARACTERS_LIMIT).collect::<String>();
        _ = &self.channel_id.say(&self.http, message).await.unwrap();
    }
}
