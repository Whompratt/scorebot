use crate::database::{add_guild, add_scoreboard, get_boards_from_db, Scoreboard};
use poise::serenity_prelude as serenity;
use std::{collections::HashMap, env::var, sync::Mutex, time::Duration};
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
use sqlx::{
    postgres::PgPoolOptions,
    {Pool, Postgres},
};

pub struct Data {
    pub pool: Pool<Postgres>,
}

use dotenv::dotenv;
use log::{error, info};

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn new_board(
    ctx: Context<'_>,
    #[description = "Scoreboard name"] scoreboard_name: String,
    #[description = "Scoreboard description"] scoreboard_description: Option<String>,
) -> Result<(), Error> {
    // 1. Gather guild id and guild name from the message context
    // 2. Register guild in guilds db if it doesn't exist
    // 3. Check if scoreboard already exists in db using guild_id and scoreboard_name
    // 4. If scoreboard doesn't exist, create scoreboard in scoreboards db
    let guild_id = ctx.guild_id().unwrap().0;
    let guild_name = ctx.guild().unwrap().name;

    // Register guild in guilds db if it doesn't exist
    match add_guild(&ctx.data().pool, guild_id, guild_name).await {
        Ok(_) => {
            info!("Guild registered");
        }
        Err(err) => {
            error!("Error registering guild: {:?}", err);
            poise::say_reply(ctx, "Error registering guild, please contact bot admin.").await?;
        }
    };

    match add_scoreboard(
        &ctx.data().pool,
        guild_id,
        scoreboard_name,
        scoreboard_description,
    )
    .await
    {
        Ok(response) => {
            info!("{}", response);
            poise::say_reply(ctx, response).await?;
        }
        Err(_err) => {
            poise::say_reply(
                ctx,
                "Error registering new scoreboard, please contact bot admin.",
            )
            .await?;
        }
    };

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn get_boards(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0;

    let scoreboard: Vec<Scoreboard> = get_boards_from_db(&ctx.data().pool, guild_id).await?;

    poise::send_reply(ctx, |f| {
        f.embed(|e| {
            e.title("Scoreboards")
                .description("Here are the scoreboards for your server:")
                .fields(
                    scoreboard
                        .iter()
                        .map(|scoreboard| {
                            (
                                scoreboard.scoreboard_name.clone(),
                                scoreboard.scoreboard_description.clone(),
                            )
                        })
                        .collect::<Vec<(&String, &String)>>(),
                )
        })
    })
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

pub async fn start_bot(pool: Pool<Postgres>) {
    dotenv().ok();

    let options = poise::FrameworkOptions {
        commands: vec![register(), new_board(), get_boards()],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .token(var("DISCORD_TOKEN").expect("token"))
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move { Ok(Data { pool: pool }) })
        })
        .options(options)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        );

    framework.run().await.unwrap();
}
