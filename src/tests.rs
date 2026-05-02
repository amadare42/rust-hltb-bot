#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::api_client::*;
    use crate::formatting::*;
    use std::fs;

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
}