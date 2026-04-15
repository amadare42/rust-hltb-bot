use crate::api_client::HltbApiClient;
use crate::retrieval_flow::RetrievalFlow;
use std::io;
use std::io::{stdout, Write};

pub async fn run_cli() {
    let mut hltb_api = HltbApiClient::new();

    loop {
        println!("Enter query: ");
        let _=stdout().flush();

        let mut query = String::new();
        io::stdin()
            .read_line(&mut query)
            .expect("Failed to read line");
        let query = query.trim().to_string();
        if query.is_empty() {
            break;
        }

        let mut flow = RetrievalFlow::new(&mut hltb_api, &query);
        println!("Fetching initial message...");

        let initial = flow.get_initial_msg().await;

        if initial.is_err() {
            println!("Failed to get initial message: {:?}", initial.err());
            continue;
        }

        println!("Initial message:\n{}", initial.unwrap());

        let final_msg = flow.get_final_msg().await;
        if final_msg.is_err() {
            println!("Failed to get final message: {:?}", final_msg.err());
            continue;
        }

        println!("Final message:\n{}", final_msg.unwrap());
    }
}
