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
    let mut rng = rand::thread_rng();
    let input = r":trap.large:\$\\color\{#927500\}\{\\sf\\bf \{東京[工業芸科学海洋術農都立医歯理心魂情報環境数物化宗教文神聖皇修帝音薬国]{2}大学[デジタルアナログ]{4}創作同好会[traana]{3}P\}\}\$";
    let expr = parser::parse(input).unwrap();
    println!("{:?}", expr.generate(&mut rng));

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
