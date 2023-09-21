#[derive(Debug, serde::Deserialize)]
pub struct Token {
    pub key: String,
    pub value: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Identity {
    pub key: String,
    pub value: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct GetIdsRequest {
    pub tokens: Vec<Token>,
    pub identities: Vec<Identity>,
}

#[derive(Debug, serde::Serialize)]
pub struct GetIdsResponse {
    // list of expired tokens so that main service can
    // todo: is there any way to refresh token, or need to check when can this token would be expire
    expired_token_cookies: Vec<String>,
    // send only one success, because we are checking as OR
    // or no success
    // note: telling which identity got success
    success: Option<Identity>,
}

#[derive(Debug, serde::Serialize)]
pub struct GetIdsError {}

pub async fn get_identities(req: GetIdsRequest) -> Result<GetIdsResponse, GetIdsError> {
    Ok(GetIdsResponse {
        expired_token_cookies: vec![],
        success: None,
    })
}
