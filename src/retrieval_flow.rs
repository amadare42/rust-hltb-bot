use std::error::Error;
use crate::api_client::HltbApiClient;
use crate::formatting::{format_msg, get_placeholder};
use crate::model::Entry;

pub struct RetrievalFlow<'a> {
    pub hltb_api: &'a mut HltbApiClient,
    pub entries: Vec<Entry>,
    pub query: String,
    pub initial_msg: String,
}

impl<'a> RetrievalFlow<'a> {
    pub fn new(hltb_api: &'a mut HltbApiClient, query: &str) -> Self {
        Self {
            hltb_api,
            entries: vec![],
            query: query.to_string(),
            initial_msg: String::new(),
        }
    }

    pub async fn get_initial_msg(&mut self) -> Result<String, Box<dyn Error>> {
        self.entries = self.hltb_api.fetch_entries(&self.query).await?;
        self.initial_msg = format_msg(&self.entries);
        Ok(self.initial_msg.clone())
    }
    
    pub async fn get_final_msg(&mut self) -> Result<String, Box<dyn Error>> {
        // Resolve steam URLs concurrently and keep them keyed by HLTB id.
        let urls_by_hltb_id = self.hltb_api.fetch_steam_urls_for_entries(&self.entries).await?;

        let mut updated_msg_text = self.initial_msg.clone();

        self.entries.iter().for_each(|entry| {
            let placeholder = get_placeholder(entry.hltb_id);

            if let Some(Some(url)) = urls_by_hltb_id.get(&entry.hltb_id) {
                let replacement = format!(" [🔗Steam]({})", url);
                updated_msg_text = updated_msg_text.replace(&placeholder, &replacement);
            } else {
                updated_msg_text = updated_msg_text.replace(&placeholder, "");
            }
        });

        Ok(updated_msg_text)
    }
}