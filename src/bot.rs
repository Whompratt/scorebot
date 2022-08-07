use poise::serenity_prelude as serenity;
use std::{collections::HashMap, env::var, sync::Mutex, time::Duration};
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {}

use dotenv::dotenv;
use log::{error, info};
// use serenity::{
//     async_trait,
//     framework::{
//         standard::{
//             macros::{command, group},
//             CommandResult,
//             StandardFramework
//         },
//     },
//     model::{
//         channel::Message,
//         gateway::Ready,
//         guild::Guild,
//     },
//     prelude::*,
//     Client,
// };

#[poise::command(slash_command, prefix_command)]
async fn new_board(
    ctx: Context<'_>,
    #[description = "Scoreboard name"] scoreboard_name: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;
    let guild_name = ctx.guild().unwrap().name;

    ctx.say(format!(
        "Creating scoreboard '{}' in '{}'",
        scoreboard_name, guild_name
    ))
    .await?;

    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            println!("Error in command '{}': {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

pub async fn start_bot() {
    dotenv().ok();

    let options = poise::FrameworkOptions {
        commands: vec![new_board(), register()],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        listener: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!("Got an event in listener: {:?}", event.name());
                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .token(var("DISCORD_TOKEN").expect("token"))
        .user_data_setup(
            move |_ctx, _ready, _framework| Box::pin(async move {
                Ok(Data {})
            })
        )
        .options(options)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        );

    framework.run().await.unwrap();
}
