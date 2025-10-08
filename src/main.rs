mod commands;
mod conversions;

use poise::serenity_prelude as serenity;
use tracing::{debug, error, info};

type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(unused)]
type Context<'a> = poise::Context<'a, Data, Error>;
const COMMAND_PREFIX: &str = "diabot ";

pub struct Data {}

#[tokio::main]
async fn main() {
    // Setup logging
    tracing_subscriber::fmt::init();
    let token = dotenvy::var("DISCORD_TOKEN").expect("Missing `DISCORD_TOKEN` env var.");
    info!("Starting Diabot");

    // Setup framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::convert::convert(),
                // Add commands here
            ],
            on_error: |error| Box::pin(on_error(error)),
            pre_command: |ctx| {
                Box::pin(async move {
                    debug!("Executing command {}...", ctx.command().qualified_name);
                })
            },
            post_command: |ctx| {
                Box::pin(async move {
                    debug!("Executed command {}!", ctx.command().qualified_name);
                })
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(COMMAND_PREFIX.into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                info!("Logged in as {}", ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let intents = serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Failed to build Serenity client");

    // Start bot
    client.start().await.expect("Client failed");
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            panic!("Failed to start bot: {:?}", error);
        }
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error);
        }
        other => {
            if let Err(e) = poise::builtins::on_error(other).await {
                error!("Error while handling error: {}", e);
            }
        }
    }
}
