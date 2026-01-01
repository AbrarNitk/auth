#[tracing::instrument(name = "oidc::service::signin-url", skip_all)]
pub async fn signin_url(
    ctx: &base::Ctx,
    client_info: &base::ReqClientInfo,
    oidc_partner: &str,
    next_url: &Option<String>,
) -> Result<AuthNLoginUrlResponse, LoginError> {
    // idp-connector must be available in the dex configuration for the org-domain
    ctx.dex
        .connectors
        .iter()
        .find(|c| c.id.eq(oidc_partner))
        .ok_or_else(|| LoginError::IDPConnectorNotFound {
            name: oidc_partner.to_owned(),
            message: format!("no `idp-connector` found"),
        })?;

    todo!()
}

#[derive(Debug, serde::Serialize)]
pub struct AuthNLoginUrlResponse {
    pub uri: String,
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("IDPConnectorNotFound: {name}")]
    IDPConnectorNotFound { name: String, message: String },
}

//////////////////////////
////// Service Layer /////
/////////////////////////

#[derive(Debug)]
pub struct AuthorizedURLReq {
    pub dex_config: base::ctx::settings::dex::DexSettings,
    pub partner_name: String,
    pub next_url: Option<String>,
    pub client_ip: Option<String>,
    pub client_user_agent: Option<String>,
}

pub struct LoginURLBuilder {
    pub state_manager: super::state::AuthStateManager,
}
