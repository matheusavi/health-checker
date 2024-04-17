pub async fn get(url: &str) -> Result<bool, String> {
    let response = reqwest::get(url).await;

    let response = match response {
        Ok(resp) => resp,
        Err(e) => return Err(e.to_string()),
    };

    match response.error_for_status_ref() {
        Ok(_) => Ok(true),
        Err(_) => match response.text().await {
            Ok(body) => Err(body),
            Err(e) => Err(e.to_string()),
        },
    }
}
