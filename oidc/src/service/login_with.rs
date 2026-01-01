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

    // build the auth request
    let authorized_url_req = AuthorizedURLReq {
        dex_config: ctx.dex.clone(),
        partner_name: oidc_partner.to_owned(), // mapping oidc partner to connector-id, must be connector-id
        next_url: next_url.clone(), //todo: may be put as the optional only and handle the default at the callback
        client_ip: client_info.ip.clone(),
        client_user_agent: client_info.user_agent.clone(),
    };

    let authorized_url_builder = LoginURLBuilder::new(ctx.redis_pool.clone());
    let authorized_url = authorized_url_builder.build_url(authorized_url_req).await?;

    Ok(AuthNLoginUrlResponse {
        uri: authorized_url,
    })
}

#[derive(Debug, serde::Serialize)]
pub struct AuthNLoginUrlResponse {
    pub uri: String,
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("IDPConnectorNotFound: {name}")]
    IDPConnectorNotFound { name: String, message: String },
    #[error("LoginUrlError: {}", _0)]
    LoginUrlError(#[from] LoginURLError),
}

impl base::response::errors::ResponseError for LoginError {
    fn error_code(&self) -> base::response::errors::ErrorCode {
        match self {
            // Self::DBError(_) => base::errors::ErrorCode::InternalServerError,
            Self::IDPConnectorNotFound { .. } => {
                base::response::errors::ErrorCode::InternalServerError
            }
            Self::LoginUrlError(_) => base::response::errors::ErrorCode::InternalServerError, // Self::CallbackHandler(_) => base::errors::ErrorCode::InternalServerError,
                                                                                              // Self::AuthFlow(_) => base::errors::ErrorCode::InternalServerError,
        }
    }

    fn message(&self) -> String {
        match self {
            // Self::HostNotFound(_) => "expected to have `host` header in request".to_string(),
            // Self::DBError(_) => "database error occurred".to_string(),
            Self::IDPConnectorNotFound { .. } => "IDP connector not found".to_string(),
            Self::LoginUrlError(_) => "url build error".to_owned(), // Self::CallbackHandler(_) => "Callback Handler Error".to_owned(),
                                                                    // Self::AuthFlow(_) => "Auth Flow Callback Error".to_owned(),
        }
    }
}

//////////////////////////
////// Service Layer /////
/////////////////////////

use openidconnect::{
    ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope,
    core::{CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
};
use reqwest::Client as HttpClient;

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

impl LoginURLBuilder {
    pub fn new(redis_pool: base::RedisPool) -> Self {
        Self {
            state_manager: super::state::AuthStateManager::new(redis_pool),
        }
    }

    #[tracing::instrument(name = "sign-in::build-url", skip_all)]
    pub async fn build_url(&self, req: AuthorizedURLReq) -> Result<String, LoginURLError> {
        tracing::info!(
            msg = "sign-url::build-req",
            oidc_partner = req.partner_name,
            client_ip = req.client_ip,
            client_user_agent = req.client_user_agent
        );

        let dex_config = req.dex_config;

        // Handling the state for the login flow
        let auth_state = super::state::AuthState::new(
            &req.partner_name,
            &req.next_url,
            10 * 60, // time to live for the state in secs
            req.client_ip,
            req.client_user_agent,
        );

        // create the signed state
        let signed_state =
            super::state::SignedState::new(auth_state.user_session_id.as_str(), "todo:secret")?;
        let signed_state_encoded = signed_state.encode()?;

        // IDP Client: For Now we are using DEX IDP
        let http_client = HttpClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| LoginURLError::BuildRequestClientError(error))?;

        // Fetch IDP metadata
        let issuer_url = IssuerUrl::new(dex_config.issuer_url).map_err(|error| {
            LoginURLError::BuildIssueURLError(format!("issuer-url error: {}", error))
        })?;
        let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &http_client)
            .await
            .map_err(|error| {
                LoginURLError::FetchProviderMetadataError(format!(
                    "provider metadata error: {}",
                    error
                ))
            })?;

        // IDP Client using the provider metadata
        let client_id = ClientId::new(dex_config.client_id.to_owned());
        let client_secret = ClientSecret::new(dex_config.client_secret.to_owned());

        let client =
            CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
                .set_redirect_uri(RedirectUrl::new(dex_config.callback_url.clone()).map_err(
                    |error| {
                        LoginURLError::BuildRedirectURLError(format!(
                            "redirect-url error: {}",
                            error
                        ))
                    },
                )?);
        // Generate PKCE challenge from verifier
        let pkce_verifier = PkceCodeVerifier::new(auth_state.code_verifier.clone());
        let pkce_challenge = PkceCodeChallenge::from_code_verifier_sha256(&pkce_verifier);

        // build the url
        let scopes: Vec<_> = dex_config
            .scopes
            .into_iter()
            .map(|s| Scope::new(s))
            .collect();

        // Prepare state and nonce values for closures
        let state_value = signed_state_encoded;
        let nonce_value = auth_state.nonce.clone();

        tracing::info!(
            msg = "generating-authorization-url",
            user_session_id = &auth_state.user_session_id,
            redirect_uri = &dex_config.callback_url
        );

        let url_request = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                move || CsrfToken::new(state_value.clone()),
                move || Nonce::new(nonce_value.clone()),
            )
            .add_scopes(scopes)
            .set_pkce_challenge(pkce_challenge)
            .add_extra_param("connector_id", req.partner_name) // todo: oidc partner name to connector name mapping, connector id may differ
            .add_prompt(CoreAuthPrompt::Login)
            .add_prompt(CoreAuthPrompt::SelectAccount);

        let (auth_url, _state, _nonce) = url_request.url();

        // store the state in the cache for later use in the callback
        self.state_manager.store(&auth_state).await?;

        tracing::info!(
            msg = "state-stored-in-cache",
            user_session_id = &auth_state.user_session_id
        );

        tracing::info!(
            msg = "authorization-url-generated",
            url = %auth_url,
        );

        Ok(auth_url.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoginURLError {
    #[error("BuildIssueURLError: {0}")]
    BuildIssueURLError(String),
    #[error("BuildRequestClientError: {0}")]
    BuildRequestClientError(#[from] reqwest::Error),
    #[error("FetchProviderMetadataError: {0}")]
    FetchProviderMetadataError(String),
    #[error("BuildRedirectURLError: {0}")]
    BuildRedirectURLError(String),
    #[error("AuthStateError: {}", _0)]
    AuthStateError(#[from] super::state::AuthStateError),
}
