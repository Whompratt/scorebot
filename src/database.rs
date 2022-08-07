use dotenv::dotenv;
use log::{debug, error};
use std::env;
use tokio::task::JoinHandle;
use tokio_postgres::{Client, Error, NoTls};

// This function creates a connection to the postgres database, and then spawns a tokio task to run the connection.
pub async fn initalize_db() -> Result<(tokio_postgres::Client, JoinHandle<()>), Error> {
    dotenv().ok();

    let postgres_password = env::var("POSTGRESQL_PASSWORD").expect("postgresql password");
    let postgres_user = env::var("POSTGRESQL_USER").expect("postgresql user");
    let postgres_dbname = env::var("POSTGRESQL_DBNAME").expect("postgresql dbname");

    // Connect to the DB
    match tokio_postgres::connect(
        &format!(
            "host=localhost port=5432 user={user} password={password} dbname={dbname}",
            user = postgres_user,
            password = postgres_password,
            dbname = postgres_dbname
        ),
        NoTls,
    )
    .await
    {
        Ok((client, connection)) => {
            // Spawn tokio thread for connection
            let join_handle = tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("{:?}", e);
                }
            });

            return Ok((client, join_handle));
        }
        Err(err) => {
            error!("Error connecting to the database: {:?}", err);
            return Err(err);
        }
    };
}

// This function uses tokio_postgres to connect to the scorebot database and creates the required tables if they don't exist
// The tables in question are:
// Guilds - guild_id, guild_name
// Scoreboards - guild_id, scoreboard_id, scoreboard_name
// Scores - scoreboard_id, user_id, score
pub async fn create_tables(client: &Client) -> Result<(), Error> {
    let guild_table_query =
        "CREATE TABLE IF NOT EXISTS guilds (guild_id bigint PRIMARY KEY, guild_name text NOT NULL)";
    let scoreboard_table_query =
        "CREATE TABLE IF NOT EXISTS scoreboards (guild_id bigint NOT NULL, scoreboard_id bigint NOT NULL, scoreboard_name text NOT NULL, PRIMARY KEY (guild_id, scoreboard_id))";
    let scores_table_query =
        "CREATE TABLE IF NOT EXISTS scores (scoreboard_id bigint NOT NULL, user_id bigint NOT NULL, score bigint NOT NULL, PRIMARY KEY (scoreboard_id, user_id))";

    // Create the tables if they don't exist
    match client.execute(guild_table_query, &[]).await {
        Ok(_) => {
            debug!("Guild table created");
        }
        Err(err) => {
            error!("Error creating guild table: {:?}", err);
            return Err(err);
        }
    }
    match client.execute(scoreboard_table_query, &[]).await {
        Ok(_) => {
            debug!("Scoreboard table created");
        }
        Err(err) => {
            error!("Error creating scoreboard table: {:?}", err);
            return Err(err);
        }
    }
    match client.execute(scores_table_query, &[]).await {
        Ok(_) => {
            debug!("Scores table created");
        }
        Err(err) => {
            error!("Error creating scores table: {:?}", err);
            return Err(err);
        }
    }

    return Ok(());
}

pub async fn register_new_guild(client: &Client) -> Result<(), Error> {
    return Ok(());
}
