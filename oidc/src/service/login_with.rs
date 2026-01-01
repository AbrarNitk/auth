pub async fn signin_url(
    ctx: &base::Ctx,
    client_info: &base::ReqClientInfo,
    oidc_partner: &str,
    next_url: &Option<String>,
) -> Result<AuthNLoginUrlResponse, LoginError> {
    todo!()
}

#[derive(Debug, serde::Serialize)]
pub struct AuthNLoginUrlResponse {
    pub uri: String,
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {}
