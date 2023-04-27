use std::str::FromStr;

use traq_ws_bot::{
    openapi::{
        self,
        models::{PostBotActionJoinRequest, PostBotActionLeaveRequest, PostMessageRequest},
    },
    utils::create_configuration,
};

use crate::config::{BOT_ACCESS_TOKEN, BOT_ID};

pub async fn send_message(channel_id: &str, text: &str, emb: bool) -> anyhow::Result<()> {
    let configuration = create_configuration(&*BOT_ACCESS_TOKEN);
    let res = openapi::apis::message_api::post_message(
        &configuration,
        channel_id,
        Some(PostMessageRequest {
            content: text.to_string(),
            embed: Some(emb),
        }),
    )
    .await?;

    log::debug!("Response: {:?}", res);

    Ok(())
}

pub async fn join_channel(channel_id: &str) -> anyhow::Result<()> {
    let configuration = create_configuration(&*BOT_ACCESS_TOKEN);
    openapi::apis::bot_api::let_bot_join_channel(
        &configuration,
        BOT_ID,
        Some(PostBotActionJoinRequest {
            channel_id: uuid::Uuid::from_str(channel_id)?,
        }),
    )
    .await?;

    Ok(())
}

pub async fn leave_channel(channel_id: &str) -> anyhow::Result<()> {
    let configuration = create_configuration(&*BOT_ACCESS_TOKEN);
    openapi::apis::bot_api::let_bot_leave_channel(
        &configuration,
        BOT_ID,
        Some(PostBotActionLeaveRequest {
            channel_id: uuid::Uuid::from_str(channel_id)?,
        }),
    )
    .await?;

    Ok(())
}
