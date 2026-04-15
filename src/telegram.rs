use std::env;
use std::error::Error;

use crate::api_client::*;
use crate::formatting::*;

use frankenstein::*;
use serde_json::Value;
use crate::retrieval_flow::RetrievalFlow;

pub struct TelegramBot {
    tg_api: Api,
    hltb_api: HltbApiClient,
}

impl TelegramBot {
    pub fn new() -> Self {
        let tg_api = create_api();
        let hltb_api = HltbApiClient::new();
        Self { tg_api, hltb_api }
    }

    pub async fn run_polling(&mut self) -> Result<(), Box<dyn Error>> {
        self.poll().await;
        Ok(())
    }

    async fn poll(&mut self) {
        log::info!("Running polling");
        let mut update_id: u32 = 0;
        loop {
            log::debug!("update_id: {}", update_id);
            let update_params = GetUpdatesParams::builder()
                .allowed_updates(vec![AllowedUpdate::Message, AllowedUpdate::EditedMessage])
                .offset(u32::clone(&update_id))
                .build();
            let update_rsp = self.tg_api.get_updates(&update_params);

            match update_rsp {
                Ok(rsp) => {
                    for update in rsp.result {
                        update_id = update.update_id + 1;
                        self.handle_update(update).await;
                    }
                }
                Err(err) => {
                    log::error!("{:?}", err)
                }
            }
        }
    }

    async fn handle_update(&mut self, update: Update) -> Option<Message> {
        if let UpdateContent::Message(message) = update.content {
            return self.respond(message).await.unwrap();
        }

        if let UpdateContent::EditedMessage(message) = update.content {
            return self.respond(message).await.unwrap();
        }

        None
    }

    async fn respond(&mut self, msg: Message) -> Result<Option<Message>, Box<dyn Error>> {
        let query = match msg.text {
            None => return Ok(None),
            Some(text) => text,
        };
        let mut flow = RetrievalFlow::new(&mut self.hltb_api, &query);

        let msg_text = &flow.get_initial_msg().await?;
        let initial_msg = SendMessageParams::builder()
            .chat_id(i64::clone(&msg.chat.id))
            .reply_to_message_id(msg.message_id)
            .text(msg_text)
            .parse_mode(
                #[allow(deprecated)]
                ParseMode::Markdown,
            )
            .build();
        log::debug!("-- sending message\n{}\n--", msg_text);
        let msg_rsp = self.tg_api.send_message(&initial_msg)?;

        // Resolve steam URLs concurrently and keep them keyed by HLTB id.
        let msg_text = flow.get_final_msg().await?;

        let updated_msg = EditMessageTextParams::builder()
            .chat_id(i64::clone(&msg.chat.id))
            .message_id(msg_rsp.result.message_id)
            .text(&msg_text)
            .parse_mode(
                #[allow(deprecated)]
                ParseMode::Markdown,
            )
            .build();
        log::debug!("-- sending updated message\n{}\n--", &msg_text);
        self.tg_api.edit_message_text(&updated_msg)?;

        Ok(Some(msg_rsp.result))
    }

    pub async fn handle_msg_from_value(&mut self, value: Value) -> Option<Message> {
        let update_content: Update = serde_json::from_value(value).unwrap();
        let rsp = self.handle_update(update_content).await;
        rsp
    }
}

pub fn register_webhook(url: &str) -> Result<MethodResponse<bool>, api::Error> {
    let params = SetWebhookParams::builder()
        .url(url)
        .allowed_updates(vec![AllowedUpdate::Message, AllowedUpdate::EditedMessage])
        .build();

    let rsp = create_api().set_webhook(&params)?;
    log::info!("{:?}", rsp);
    Ok(rsp)
}

pub fn unregister_webhook() -> Result<MethodResponse<bool>, api::Error> {
    let params = DeleteWebhookParams::builder().build();

    let rsp = create_api().delete_webhook(&params)?;
    log::info!("{:?}", rsp);
    Ok(rsp)
}

fn create_api() -> Api {
    let key = env::var("API_KEY").expect("API_KEY is missing in env variables.");
    Api::new(&key)
}
