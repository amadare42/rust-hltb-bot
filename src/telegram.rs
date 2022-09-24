use std::error::Error;
use std::env;

use crate::api_client::*;
use crate::formatting::*;

use frankenstein::*;
use serde_json::Value;

pub async fn run_polling() -> Result<(), Box<dyn Error>> {
    poll(&create_api()).await;

    return Ok(())
}

pub async fn handle_msg_from_value(value: Value) -> Option<Message> {
    let update_content: Update = serde_json::from_value(value).unwrap();
    let api = create_api();
    let rsp = handle_update(&api, update_content).await;
    rsp
}

pub fn register_webhook(url: &str) -> Result<MethodResponse<bool>, frankenstein::api::Error> {
    let params = SetWebhookParams::builder()
        .url(url)
        .allowed_updates(vec![AllowedUpdate::Message, AllowedUpdate::EditedMessage])
        .build();

    let rsp = create_api().set_webhook(&params)?;
    log::info!("{:?}", rsp);
    Ok(rsp)
}

pub fn unregister_webhook() -> Result<MethodResponse<bool>, frankenstein::api::Error> {
    let params = DeleteWebhookParams::builder().build();

    let rsp = create_api().delete_webhook(&params)?;
    log::info!("{:?}", rsp);
    Ok(rsp)
}


fn create_api() -> Api {
    let key = env::var("API_KEY")
        .expect("API_KEY is missing in env variables.");
    Api::new(&key)
}

async fn poll(api: &Api) {
    log::info!("Running polling");
    let mut update_id: u32 = 0;
    loop {
        log::debug!("update_id: {}", update_id);
        let update_params = GetUpdatesParams::builder()
            .allowed_updates(vec![AllowedUpdate::Message, AllowedUpdate::EditedMessage])
            .offset(u32::clone(&update_id))
            .build();
        let update_rsp = api.get_updates(&update_params);

        match update_rsp {
            Ok(rsp) => {
                for update in rsp.result {
                    update_id = update.update_id + 1;
                    handle_update(&api, update).await;
                }
            }
            Err(err) => {
                log::error!("{:?}", err)
            }
        }
    }
}

async fn handle_update(api: &Api, update: Update) -> Option<Message> {
    if let UpdateContent::Message(message) = update.content {
        return respond(&api, message).await.unwrap();
    }

    if let UpdateContent::EditedMessage(message) = update.content {
        return respond(&api, message).await.unwrap();
    }

    None
}

async fn respond(api: &Api, msg: Message) -> Result<Option<Message>, Box<dyn Error>> {
    let query = match msg.text {
        None => return Ok(None),
        Some(text) => text
    };
    let entries = fetch_entries(&query).await?;
    let msg_text = format_msg(&entries);

    let initial_msg = SendMessageParams::builder()
        .chat_id(i64::clone(&msg.chat.id))
        .reply_to_message_id(msg.message_id)
        .text(&msg_text)
        .parse_mode(#[allow(deprecated)] ParseMode::Markdown)
        .build();
    log::debug!("-- sending message\n{}\n--", msg_text);
    let msg_rsp = api.send_message(&initial_msg)?;

    Ok(Some(msg_rsp.result))
}
