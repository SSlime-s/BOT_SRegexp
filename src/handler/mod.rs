use std::{convert::identity, sync::Arc};

use anyhow::Result;
use traq_ws_bot::events::{common::Message, payload};

use crate::{
    config::Resource,
    generator::Generate,
    model::{
        api::{join_channel, leave_channel, send_message},
        db,
    },
    parser,
};

/// like !{\"type\":\"user\",\"raw\":\"@BOT_STimer\",\"id\":\"d352688f-a656-4444-8c5f-caa517e9ea1b\"}
const MENTION_REGEX: &str =
    r#"!\{"type":"user","raw":"(?:[^\\"]|\\.)+","id":"d352688f-a656-4444-8c5f-caa517e9ea1b"\}"#;

const SPECIAL_MESSAGE_REGEX: &str =
    r#"!\{"type":"(user|channel|group)","raw":"(?P<raw>(?:[^\\"]|\\.)+)","id":"(?:[^\\"]|\\.)+"\}"#;

const LENGTH_LIMIT: usize = 3000;

async fn generate_text(regexp: &str) -> Result<String, String> {
    let parsed = parser::parse(regexp).map_err(|e| format!("Failed to parse: {:?}", e))?;
    let mut rng = rand::thread_rng();
    let text = parsed
        .generate(&mut rng)
        .map_err(|e| format!("Failed to generate: {:?}", e))?;
    if text.len() > LENGTH_LIMIT {
        Err(format!(
            "Generated text is too long: {} > {}",
            text.len(),
            LENGTH_LIMIT
        ))
    } else {
        Ok(text)
    }
}

async fn message_like_handler(message: Message, resource: Arc<Resource>) {
    log::debug!("Received message: {:?}", message);
    if message.user.bot {
        return;
    }

    // TODO: handle mentioned message
    let content = message.text;

    let Ok(command) = parse_command(&content)
        else {
            todo!()
        };

    match command {
        Command::RandRegexp(regexp) => {
            let text = generate_text(&regexp).await.unwrap_or_else(identity);
            let res = send_message(&message.channel_id, &text, false).await;
            if let Err(e) = res {
                log::error!("Failed to send message: {:?}", e);
            }
        }
        Command::Save { key, value } => {
            let user_id = message.user.id;
            let user_name = message.user.name;
            let pool = resource.clone();
            let result = db::save(&pool, &key, &value, &user_id, &user_name).await;
            let text = match result {
                Ok(_) => {
                    format!("Saved: {} => {}", key, value)
                }
                Err(e) => {
                    // if key duplicated error (code: 1062)
                    if e.as_database_error().map_or(false, |e| {
                        e.code().map(|x| x.to_string()) == Some("1062".to_string())
                    }) {
                        format!("Key \"{}\" is already exists", key)
                    } else {
                        format!("Failed to save: {}", e)
                    }
                }
            };
            let res = send_message(&message.channel_id, &text, false).await;
            if let Err(e) = res {
                log::error!("Failed to send message: {:?}", e);
            }
        }
        Command::Call(key) => {
            let pool = resource.clone();
            let result = db::get(&pool, &key).await;
            let text = match result {
                Ok(Some(value)) => generate_text(&value).await.unwrap_or_else(identity),
                Ok(None) => {
                    format!("Key \"{}\" is not found", key)
                }
                Err(e) => {
                    format!("Failed to get from database: {}", e)
                }
            };
            let res = send_message(&message.channel_id, &text, false).await;
            if let Err(e) = res {
                log::error!("Failed to send message: {:?}", e);
            }
        }
        Command::Remove(key) => {
            let pool = resource.clone();
            let user_id = message.user.id;
            let result = db::remove(&pool, &key, &user_id).await;
            let text = match result {
                Ok(true) => {
                    format!("Removed: {}", key)
                }
                Ok(false) => {
                    format!("Key \"{}\" is not found", key)
                }
                Err(e) => {
                    format!("Failed to remove: {}", e)
                }
            };

            let res = send_message(&message.channel_id, &text, false).await;
            if let Err(e) = res {
                log::error!("Failed to send message: {:?}", e);
            }
        }
        Command::Join => {
            let res = join_channel(&message.channel_id).await;
            if let Err(e) = res {
                log::error!("Failed to join channel: {:?}", e);
            }
        }
        Command::Leave => {
            let res = leave_channel(&message.channel_id).await;
            if let Err(e) = res {
                log::error!("Failed to leave channel: {:?}", e);
            }
        }
    }
}

pub async fn on_message_created(payload: payload::MessageCreated, resource: Arc<Resource>) {
    log::debug!("Received message created: {:?}", payload);
    let message = payload.message;
    message_like_handler(message, resource).await;
}
pub async fn on_direct_message_created(
    payload: payload::DirectMessageCreated,
    resource: Arc<Resource>,
) {
    log::debug!("Received message updated: {:?}", payload);
    let message = payload.message;
    message_like_handler(message, resource).await;
}

#[derive(Clone, Debug)]
pub enum Command {
    RandRegexp(String),
    Save { key: String, value: String },
    Call(String),
    Remove(String),
    Join,
    Leave,
}

pub fn parse_command(input: &str) -> Result<Command> {
    let content = input.trim();
    let splitted = content.split_whitespace().collect::<Vec<_>>();

    if splitted.is_empty() || !splitted[0].starts_with("/") {
        return Err(anyhow::anyhow!("/ で始まるコマンドが必須です"));
    }

    match &splitted[0][1..] {
        command @ ("regex" | "regexp" | "rand" | "random" | "randregex") => {
            let rest = content.trim_start_matches(&format!("/{command}")).trim();
            Ok(Command::RandRegexp(rest.to_string()))
        }
        command @ ("save" | "memory") => {
            if splitted.len() < 2 {
                return Err(anyhow::anyhow!("key が必須です"));
            }

            let rest = content.trim_start_matches(&format!("/{command}")).trim();
            let key = splitted[1].to_string();
            let value = rest.trim_start_matches(&key).trim().to_string();
            Ok(Command::Save { key, value })
        }
        command @ ("call" | "load") => {
            if splitted.len() < 2 {
                return Err(anyhow::anyhow!("key が必須です"));
            }
            if splitted.len() > 2 {
                return Err(anyhow::anyhow!("key に空白を含めることはできません"));
            }

            let rest = content.trim_start_matches(&format!("/{command}")).trim();
            Ok(Command::Call(rest.to_string()))
        }
        command @ ("remove" | "delete" | "forget") => {
            if splitted.len() < 2 {
                return Err(anyhow::anyhow!("key が必須です"));
            }
            if splitted.len() > 2 {
                return Err(anyhow::anyhow!("key に空白を含めることはできません"));
            }
            let rest = content.trim_start_matches(&format!("/{command}")).trim();
            Ok(Command::Remove(rest.to_string()))
        }
        "join" => Ok(Command::Join),
        "leave" | "bye" => Ok(Command::Leave),
        unknown => Err(anyhow::anyhow!("unknown command /{}", unknown)),
    }
}
