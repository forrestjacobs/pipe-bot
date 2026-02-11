use mockall::automock;
use serenity::all::{ActivityData, ChannelId, Context};

#[automock]
pub trait DiscordContext {
    async fn say(&self, channel_id: ChannelId, content: &str) -> serenity::Result<()>;
    fn set_activity(&self, activity: Option<ActivityData>);
}

impl DiscordContext for Context {
    async fn say(&self, channel_id: ChannelId, content: &str) -> serenity::Result<()> {
        channel_id.say(&self.http, content).await?;
        Ok(())
    }
    fn set_activity(&self, activity: Option<ActivityData>) {
        self.set_activity(activity);
    }
}
