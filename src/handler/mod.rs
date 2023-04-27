use anyhow::Result;
use traq_ws_bot::events::common::Message;

/// like !{\"type\":\"user\",\"raw\":\"@BOT_STimer\",\"id\":\"d352688f-a656-4444-8c5f-caa517e9ea1b\"}
const MENTION_REGEX: &str =
    r#"!\{"type":"user","raw":"(?:[^\\"]|\\.)+","id":"d352688f-a656-4444-8c5f-caa517e9ea1b"\}"#;

const SPECIAL_MESSAGE_REGEX: &str =
    r#"!\{"type":"(user|channel|group)","raw":"(?P<raw>(?:[^\\"]|\\.)+)","id":"(?:[^\\"]|\\.)+"\}"#;

async fn message_like_handler(message: Message, resource: ()) {
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
        _ => todo!(),
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    RandRegexp(String),
    Save(String),
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
            let rest = content.trim_start_matches(command).trim();
            Ok(Command::RandRegexp(rest.to_string()))
        }
        command @ ("save" | "memory") => {
            let rest = content.trim_start_matches(command).trim();
            Ok(Command::Save(rest.to_string()))
        }
        command @ ("call" | "load") => {
            let rest = content.trim_start_matches(command).trim();
            Ok(Command::Call(rest.to_string()))
        }
        command @ ("remove" | "delete" | "forget") => {
            let rest = content.trim_start_matches(command).trim();
            Ok(Command::Remove(rest.to_string()))
        }
        command @ "join" => Ok(Command::Join),
        command @ ("leave" | "bye") => Ok(Command::Leave),
        unknown => Err(anyhow::anyhow!("unknown command /{}", unknown)),
    }
}
