use serenity::{model::prelude::*, prelude::*};
type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

#[derive(Default)]
struct Handler {
    handler_lib: std::sync::Mutex<Option<std::sync::Arc<libloading::Library>>>,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        match &*msg.content {
            "load" => {
                *self.handler_lib.lock().unwrap() = Some(std::sync::Arc::new(unsafe {
                    libloading::Library::new("target/debug/libcommands.so").unwrap()
                }));

                msg.channel_id.say(&ctx, "Loaded module").await.unwrap();
            }
            "unload" => {
                *self.handler_lib.lock().unwrap() = None;

                msg.channel_id.say(&ctx, "Unloaded module").await.unwrap();
            }
            "ping" => {
                let handler_lib = self.handler_lib.lock().unwrap().clone();
                if let Some(handler_lib) = handler_lib {
                    type Command =
                        fn(Context, Message) -> BoxFuture<'static, Result<(), serenity::Error>>;
                    let command = unsafe { handler_lib.get::<Command>(b"ping").unwrap() };

                    command(ctx, msg).await.unwrap();
                }
            }
            _ => {}
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), serenity::Error> {
    let token = std::env::var("TOKEN").expect("expected TOKEN environment variable");
    serenity::client::ClientBuilder::new(token)
        .event_handler(Handler::default())
        .await?
        .start()
        .await
}
