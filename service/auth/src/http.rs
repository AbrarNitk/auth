pub enum HttpError {}

async fn get<R: serde::de::DeserializeOwned>(
    url: &str,
    headers: &std::collections::HashMap<String, String>,
) -> Result<R, HttpError> {
    Ok(serde_json::from_str("").unwrap())
}

async fn post<R: serde::de::DeserializeOwned>(
    url: &str,
    body: impl serde::Serialize,
    headers: &std::collections::HashMap<String, String>,
) -> Result<R, HttpError> {
    Ok(serde_json::from_str("").unwrap())
}
