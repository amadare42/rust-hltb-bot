#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::time::Duration;

    use crate::api_client::*;
    use crate::formatting::*;
    use serde_json::json;
    use std::fs;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn format_from_local() -> Result<(), Box<dyn Error>> {
        let response = fs::read_to_string("./example_search_response.json").unwrap();
        let response = serde_json::from_str(&response).unwrap();
        let entries = parse_entries_from_rsp(response, "https://example.com").unwrap();

        println!("found {} entries", entries.len());

        let msg = format_msg(&entries);
        println!("{}", msg);

        Ok(())
    }

    #[tokio::test]
    async fn format_from_api() -> Result<(), Box<dyn Error>> {
        dotenvy::dotenv().ok();
        let mut sut = HltbApiClient::new_from_env();
        let entries = sut.fetch_entries(&"skyrim").await?;

        println!("found {} entries", entries.len());

        let msg = format_msg(&entries);
        println!("{}", msg);

        Ok(())
    }

    #[tokio::test]
    async fn fetch_steam_id() -> Result<(), Box<dyn Error>> {
        dotenvy::dotenv().ok();
        let mut sut = HltbApiClient::new_from_env();
        let url = sut.fetch_steam_url(14996).await?;

        assert_eq!(url, Some("https://store.steampowered.com/app/489830/".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn fetch_entries_flow() -> Result<(), Box<dyn Error>> {
        dotenvy::dotenv().ok();
        let mut sut = HltbApiClient::new_from_env();
        let entries = sut.fetch_entries(&"skyrim").await?;
        assert_eq!(entries.len(), 5);

        Ok(())
    }

    #[test]
    fn extracts_query_from_api_gateway_querystring_parameters() {
        let value = json!({
            "lambda_rq_type": "query",
            "queryStringParameters": { "q": "skyrim" }
        });

        assert_eq!(crate::lambda::extract_query(&value), Some("skyrim"));
    }

    #[test]
    fn local_runtime_detection_uses_direct_invocation_without_aws_runtime() {
        let old_runtime_api = std::env::var("AWS_LAMBDA_RUNTIME_API").ok();
        let old_execution_env = std::env::var("AWS_EXECUTION_ENV").ok();
        std::env::remove_var("AWS_LAMBDA_RUNTIME_API");
        std::env::remove_var("AWS_EXECUTION_ENV");

        assert!(!crate::lambda::should_use_aws_runtime());

        if let Some(v) = old_runtime_api {
            std::env::set_var("AWS_LAMBDA_RUNTIME_API", v);
        }
        if let Some(v) = old_execution_env {
            std::env::set_var("AWS_EXECUTION_ENV", v);
        }
    }

    #[tokio::test]
    async fn auth_request_times_out_when_remote_stalls() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let old_domain = std::env::var("HLTB_DOMAIN_URL").ok();
        let old_init = std::env::var("HLTB_INIT_URL").ok();
        let old_find = std::env::var("HLTB_FIND_URL").ok();
        std::env::set_var("HLTB_DOMAIN_URL", format!("http://{addr}"));
        std::env::set_var("HLTB_INIT_URL", format!("http://{addr}?t=1"));
        std::env::set_var("HLTB_FIND_URL", format!("http://{addr}/api/search/site"));

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let _stream = stream;
            tokio::time::sleep(Duration::from_secs(30)).await;
        });

        let mut sut = HltbApiClient::new_from_env();
        let result = tokio::time::timeout(Duration::from_secs(25), sut.ensure_auth()).await;
        assert!(result.is_ok(), "ensure_auth should fail within the configured timeout");

        if let Some(v) = old_domain {
            std::env::set_var("HLTB_DOMAIN_URL", v);
        } else {
            std::env::remove_var("HLTB_DOMAIN_URL");
        }
        if let Some(v) = old_init {
            std::env::set_var("HLTB_INIT_URL", v);
        } else {
            std::env::remove_var("HLTB_INIT_URL");
        }
        if let Some(v) = old_find {
            std::env::set_var("HLTB_FIND_URL", v);
        } else {
            std::env::remove_var("HLTB_FIND_URL");
        }

        match result.unwrap() {
            Ok(_) => panic!("auth request should fail on a stalled upstream"),
            Err(err) => {
                let msg = err.to_string();
                assert!(msg.contains("timed out") || msg.contains("error sending request"));
            }
        }
    }
}