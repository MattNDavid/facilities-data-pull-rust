use reqwest::{Client, Error};
use serde_json::Value;

const PER_PAGE: u32 = 100;
const BASE_URL: &str = "https://api.planningcenteronline.com";

// Fetches all pages of data from a Planning Center Online API endpoint.
// `params` should be additional query params, e.g. "order=title&filter=archived"
pub async fn get_pc_data(
    endpoint: &str,
    params: &str,
    app_id: &str,
    secret: &str,
) -> Result<(Vec<Value>, Vec<Value>), Error> {
    let client = Client::new();
    let mut all_items: Vec<Value> = Vec::new();
    let mut all_includes: Vec<Value> = Vec::new();
    let mut offset: u32 = 0;
    let mut _pages_fetched = 0;
    loop {
        let query = if params.is_empty() {
            format!("per_page={PER_PAGE}&offset={offset}")
        } else {
            format!("{params}&per_page={PER_PAGE}&offset={offset}")
        };

        let url = format!("{BASE_URL}{endpoint}?{query}");

        let response = client
            .get(&url)
            .basic_auth(app_id, Some(secret))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        // Extract items and included data; if "data" is missing or not an array, break the loop
        let items = match response.get("data").and_then(|d| d.as_array()) {
            Some(data) => data.clone(),
            None => break,
        };

        let includes = response.get("included")
            .and_then(|i| i.as_array())
            .cloned()
            .unwrap_or_default();

        let fetched = items.len() as u32;
        all_items.extend(items);
        all_includes.extend(includes);
        // Planning Center returns total_count in meta; stop when we've collected everything
        let total = response
            .pointer("/meta/total_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        offset += fetched;
        if fetched < PER_PAGE || offset >= total {
            break;
        }
        _pages_fetched += 1;
    }
    
    Ok((all_items, all_includes))
}