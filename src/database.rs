use dotenv::dotenv;
use log::{debug, error};
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    Row, {Pool, Postgres},
};
use std::env;

pub struct Scoreboard {
    pub scoreboard_id: i32,
    pub guild_id: i64,
    pub scoreboard_name: String,
    pub scoreboard_description: Option<String>,
}

// This function creates a connection to the postgres database, and then spawns a tokio task to run the connection, returning both the connection as a join_handle and the client
pub async fn initalize_db() -> Result<Pool<Postgres>, sqlx::Error> {
    dotenv().ok();

    let postgresql_dbname = env::var("POSTGRESQL_DBNAME").unwrap();
    let postgresql_user = env::var("POSTGRESQL_USER").unwrap();
    let postgresql_password = env::var("POSTGRESQL_PASSWORD").unwrap();

    match PgPoolOptions::new()
        .max_connections(5)
        .connect(
            &format!(
                "postgres://localhost?dbname={POSTGRESQL_DBNAME}&user={POSTGRESQL_USER}&password={POSTGRESQL_PASSWORD}",
                POSTGRESQL_DBNAME=postgresql_dbname,
                POSTGRESQL_USER=postgresql_user,
                POSTGRESQL_PASSWORD=postgresql_password
            )
        ).await {
            Ok(pool) => Ok(pool),
            Err(err) => Err(err)
        }
}

// This function uses tokio_postgres to connect to the scorebot database and creates the required tables if they don't exist
// The tables in question are:
// Guilds - guild_id, guild_name
// Scoreboards - guild_id, scoreboard_id, scoreboard_name, scoreboard_description (optional)
// Scores - scoreboard_id, user_id, score
pub async fn create_tables(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    dotenv().ok();

    // Create the tables if they don't exist
    // Guilds
    match sqlx::query!(
        "CREATE TABLE IF NOT EXISTS guilds \
        ( \
            guild_id BIGINT PRIMARY KEY, \
            guild_name VARCHAR(255) NOT NULL \
        )",
    )
    .execute(pool)
    .await
    {
        Ok(_) => {
            debug!("Guilds table created");
        }
        Err(err) => {
            error!("Error creating guilds table: {:?}", err);
            return Err(err);
        }
    };
    // Scoreboards
    match sqlx::query!(
        "CREATE TABLE IF NOT EXISTS scoreboards \
        ( \
            scoreboard_id SERIAL PRIMARY KEY,
            guild_id BIGINT NOT NULL,
            scoreboard_name VARCHAR(255) NOT NULL, \
            scoreboard_description VARCHAR(255), \
            CONSTRAINT fk_guild_id FOREIGN KEY (guild_id) REFERENCES guilds(guild_id) \
        )"
    )
    .execute(pool)
    .await
    {
        Ok(_) => {
            debug!("Scoreboards table created");
        }
        Err(err) => {
            error!("Error creating scoreboards table: {:?}", err);
            return Err(err);
        }
    };
    // Scores
    match sqlx::query!(
        "CREATE TABLE IF NOT EXISTS scores \
        ( \
            user_id BIGINT PRIMARY KEY, \
            scoreboard_id SERIAL NOT NULL, \
            score INT NOT NULL DEFAULT 0, \
            CONSTRAINT fk_scoreboard_id FOREIGN KEY (scoreboard_id) REFERENCES scoreboards(scoreboard_id) \
        )"
    )
    .execute(pool)
    .await
    {
        Ok(_) => {
            debug!("Scores table created");
        }
        Err(err) => {
            error!("Error creating scores table: {:?}", err);
            return Err(err);
        }
    };

    return Ok(());
}

pub async fn add_guild(
    pool: &Pool<Postgres>,
    guild_id: u64,
    guild_name: String,
) -> Result<(), sqlx::Error> {
    let guild_query = format!("INSERT INTO guilds (guild_id, guild_name) VALUES ({guild_id}, '{guild_name}') ON CONFLICT (guild_id) DO NOTHING", guild_id=guild_id, guild_name=guild_name);
    match sqlx::query(&guild_query).execute(pool).await {
        Ok(_) => {
            debug!("Guild added");
        }
        Err(err) => {
            error!("Error adding guild: {:?}", err);
            return Err(err);
        }
    };

    Ok(())
}

pub async fn add_scoreboard(
    pool: &Pool<Postgres>,
    guild_id: u64,
    scoreboard_name: String,
    scoreboard_description: Option<String>,
) -> Result<String, sqlx::Error> {
    let scoreboard_check_query = format!(
        "SELECT * FROM scoreboards WHERE guild_id = {guild_id} AND scoreboard_name = '{scoreboard_name}'", guild_id=guild_id.to_string(), scoreboard_name=scoreboard_name
    );

    let scoreboard_insert_query;

    if let Some(scoreboard_description) = scoreboard_description {
        scoreboard_insert_query = format!(
            "INSERT INTO scoreboards (guild_id, scoreboard_name, scoreboard_description) VALUES ({guild_id}, '{scoreboard_name}', '{scoreboard_description}')", guild_id=guild_id.to_string(), scoreboard_name=scoreboard_name, scoreboard_description=scoreboard_description
        );
    } else {
        scoreboard_insert_query = format!(
            "INSERT INTO scoreboards (guild_id, scoreboard_name) VALUES ({guild_id}, '{scoreboard_name}')", guild_id=guild_id.to_string(), scoreboard_name=scoreboard_name
        );
    }

    match sqlx::query(&scoreboard_check_query)
        .fetch_optional(pool)
        .await
    {
        Ok(row) => {
            if row.is_none() {
                match sqlx::query(&scoreboard_insert_query).execute(pool).await {
                    Ok(_) => {
                        debug!("Scoreboard added");
                        return Ok(format!("Scoreboard {} added", scoreboard_name).to_string());
                    }
                    Err(err) => {
                        error!("Error adding scoreboard: {:?}", err);
                        return Err(err);
                    }
                }
            } else {
                debug!("Scoreboard already exists");
                return Ok(format!("Scoreboard {} already exists", scoreboard_name).to_string());
            }
        }
        Err(err) => {
            println!("Error checking scoreboard: {:?}", err);
            return Err(err);
        }
    };
}

pub async fn get_boards_from_db(
    pool: &Pool<Postgres>,
    guild_id: u64,
) -> Result<Vec<Scoreboard>, sqlx::Error> {
    let scoreboard_query = format!(
        "SELECT * FROM scoreboards WHERE guild_id = {guild_id}",
        guild_id = guild_id.to_string()
    );

    match sqlx::query(&scoreboard_query).fetch_all(pool).await {
        Ok(scoreboard_rows) => {
            let mut scoreboards: Vec<Scoreboard> = Vec::new();

            for row in scoreboard_rows {
                let scoreboard = Scoreboard {
                    scoreboard_id: row.get("scoreboard_id"),
                    guild_id: row.get("guild_id"),
                    scoreboard_name: row.get("scoreboard_name"),
                    scoreboard_description: row.get("scoreboard_description"),
                };
                scoreboards.push(scoreboard);
            }

            return Ok(scoreboards);
        }
        Err(err) => {
            error!("Error getting scoreboards: {:?}", err);
            return Err(err);
        }
    };
}
