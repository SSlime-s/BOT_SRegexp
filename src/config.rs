use std::env;

use once_cell::sync::Lazy;

/// この BOT の UUID
pub const BOT_ID: &str = "6404a6b2-6f1e-4471-bcec-8352bcf0a83a";

/// この BOT の USER ID
pub const BOT_USER_ID: &str = "0a0be82e-a9a1-4211-89c6-f7dbb0dced8c";

/// この BOT の ACCESS TOKEN
pub static BOT_ACCESS_TOKEN: Lazy<String> = Lazy::new(|| {
    dotenv::dotenv().ok();
    env::var("BOT_ACCESS_TOKEN").expect("BOT_ACCESS_TOKEN is required")
});

pub type Resource = sqlx::MySqlPool;
