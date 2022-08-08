mod bot;
mod database;

use crate::bot::start_bot;
use crate::database::{create_tables, initalize_db};
use sqlx::{Pool, Postgres};

use dotenv::dotenv;

use log::{debug, error};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    // Create the database thread
    let pool: Pool<Postgres> = match initalize_db().await {
        Ok(pool) => pool,
        Err(err) => {
            error!("Error creating database pool: {:?}", err);
            return;
        }
    };

    // Create the tables if they don't exist
    match create_tables(&pool).await {
        Ok(_) => {
            debug!("Tables created");
        }
        Err(err) => {
            panic!("Error creating tables: {:?}", err);
        }
    };

    start_bot(pool).await;
}
