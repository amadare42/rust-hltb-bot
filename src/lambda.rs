use std::fmt::Debug;
use std::time::Instant;
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{Value};
use crate::{telegram, get_bot};
use crate::api_client::HltbApiClient;
use crate::retrieval_flow::RetrievalFlow;

pub(crate) fn should_use_aws_runtime() -> bool {
    std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok()
        || std::env::var("AWS_EXECUTION_ENV").is_ok()
}

pub async fn run() -> Result<(), Error> {
    if !should_use_aws_runtime() {
        let payload = std::env::var("LOCAL_LAMBDA_EVENT")
            .or_else(|_| std::env::var("LAMBDA_EVENT"))
            .unwrap_or_else(|_| r#"{"lambda_rq_type":"query","query":"Skyrim"}"#.to_string());

        let value: Value = serde_json::from_str(&payload)?;
        let rsp = handle_rq(value).await;
        println!("{}", rsp);
        return Ok(());
    }

    let func = service_fn(handle);
    lambda_runtime::run(func).await?;
    Ok(())
}

pub(crate) fn extract_query(value: &Value) -> Option<&str> {
    if let Some(query) = value.get("query").and_then(Value::as_str) {
        if !query.is_empty() && query != "$input.params().querystring.get('q')" {
            return Some(query);
        }
    }

    if let Some(query) = value
        .get("queryStringParameters")
        .and_then(Value::as_object)
        .and_then(|params| params.get("q"))
        .and_then(Value::as_str)
    {
        return Some(query);
    }

    value.get("q").and_then(Value::as_str)
}

pub async fn handle_rq(value: Value) -> String {
    log::info!("{:?}", value);

    match value["lambda_rq_type"].as_str() {
        Some("register_webhook") => match value["url"].as_str() {
            None => "'url' property is missing".to_string(),
            Some(url) => match telegram::register_webhook(&url) {
                Ok(rsp) => {
                    log::info!("registering webhook rq succeeded {:?}", rsp);
                    to_str(&rsp)
                }
                Err(err) => {
                    log::error!("registering webhook rq failed {:?}", err);
                    to_str(&err)
                }
            }
        },
        Some("remove_webhook") => match telegram::unregister_webhook() {
            Ok(rsp) => {
                log::info!("removing webhook rq succeeded {:?}", rsp);
                to_str(&rsp)
            }
            Err(err) => {
                log::error!("removing webhook rq failed {:?}", err);
                to_str(&err)
            }
        },
        Some("query") => match extract_query(&value) {
            None => "'query' property is missing".to_string(),
            Some(query) => get_query_result(query).await.unwrap_or_else(|err| {
                log::error!("{:?}", &err);
                format!("Failed to get query result: {:?}", &err)
            }),
        },
        Some(msg_type) => format!("Unknown message type: {}", msg_type),
        _ => {
            let rsp = get_bot()
                .lock()
                .unwrap()
                .handle_msg_from_value(value).await;
            log::info!("{:?}", rsp);
            to_str(&rsp)
        }
    }
}


async fn get_query_result(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut api = HltbApiClient::new_from_env();
    let mut flow = RetrievalFlow::new(&mut api, query);
    let start = Instant::now();
    log::info!("Starting HLTB search for query {:?}", query);
    let initial_msg = &flow.get_initial_msg().await?;
    let initial_duration = start.elapsed();
    log::info!("HLTB search completed in {}ms", initial_duration.as_millis());
    let final_msg = &flow.get_final_msg().await?;
    let duration = start.elapsed();
    log::info!("HLTB Steam-link enrichment completed in {}ms", duration.as_millis());

    let page = format!(r#"
<html>
<head>
<title>HLTB Query Result</title>
</head>
<body>
<h1>Initial Message</h1>
<pre>{}</pre>
<br/>
<h1>Final Message</h1>
<pre>{}</pre>

<hr>
<p><small>initial: {}ms; total: {}ms</small></p>
</html>
"#, initial_msg, final_msg, initial_duration.as_millis(), duration.as_millis());

    Ok(page)
}

pub(crate) async fn handle(event: LambdaEvent<Value>) -> Result<String, Error> {
    let (event, _context) = event.into_parts();
    let rsp = handle_rq(event).await;
    Ok(rsp)
}


fn to_str<T>(rsp: &T) -> String
    where T : Debug + serde::ser::Serialize
{
    serde_json::to_string(rsp)
        .unwrap_or(format!("{:?}", rsp).to_string())
}


#[cfg(test)]
mod test {
    use crate::lambda::*;

    #[tokio::test]
     #[ignore = "requires WEBHOOK_URL and Telegram credentials"]
    async fn test_register_webhook() {
        let url = std::env::var("WEBHOOK_URL")
            .expect("WEBHOOK_URL env parameter should be set!");
        let json_str = format!(r#"
{{
  "lambda_rq_type": "register_webhook",
  "url": "{url}"
}}
        "#, url=url);
        let rq: Value = serde_json::from_str(&json_str).unwrap();
        println!("{:?}", handle_rq(rq).await);
    }

    #[tokio::test]
    #[ignore = "requires WEBHOOK_URL and Telegram credentials"]
    async fn test_unregister_webhook() {
        std::env::var("WEBHOOK_URL")
            .expect("WEBHOOK_URL env parameter should be set!");
        let rq: Value = serde_json::from_str(r#"
{
  "lambda_rq_type": "remove_webhook"
}
        "#).unwrap();
        println!("{:?}", handle_rq(rq).await);
    }
}