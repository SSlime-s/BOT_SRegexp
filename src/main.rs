mod config;
mod generator;
mod handler;
mod model;
mod parser;

use traq_ws_bot::builder;

use crate::{config::BOT_ACCESS_TOKEN, model::db::connect_db};
use generator::Generate;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    dotenv::from_filename(".env.local").ok();

    env_logger::init();

    log::debug!("Connecting to database...");
    let pool = connect_db().await.unwrap();
    log::debug!("Connected to database");

    let bot = builder(&*BOT_ACCESS_TOKEN)
        .insert_resource(pool)
        .on_message_created_with_resource(handler::on_message_created)
        .on_direct_message_created_with_resource(handler::on_direct_message_created)
        .build();

    log::debug!("Starting bot...");
    bot.start().await.unwrap();
}
