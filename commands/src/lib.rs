use serenity::{model::prelude::*, prelude::*};
type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

#[no_mangle]
pub fn ping(ctx: Context, msg: Message) -> BoxFuture<'static, Result<(), serenity::Error>> {
    Box::pin(async move {
        msg.channel_id.say(&ctx, "Pong!").await?;
        Ok(())
    })
}
