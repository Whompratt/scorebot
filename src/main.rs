mod bot;
mod database;

use crate::bot::start_bot;
use crate::database::create_tables;

use dotenv::dotenv;

use log::debug;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    // Create the database thread
    let (client, _connection) = match database::initalize_db().await {
        Ok((client, connection)) => (client, connection),
        Err(err) => {
            panic!("Error connecting to the database: {:?}", err);
        }
    };

    // Create the tables if they don't exist
    match create_tables(&client).await {
        Ok(_) => {
            debug!("Tables created");
        }
        Err(err) => {
            panic!("Error creating tables: {:?}", err);
        }
    };

    start_bot().await;
}
