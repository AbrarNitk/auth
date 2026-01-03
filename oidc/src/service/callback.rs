use base::ctx::settings::dex::DexSettings;
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, IssuerUrl, Nonce, PkceCodeVerifier, RedirectUrl,
    TokenResponse,
    core::{CoreClient, CoreIdTokenClaims, CoreProviderMetadata, CoreTokenResponse},
};
use reqwest::Client as HttpClient;

use crate::service::{
    login_with::LoginError,
    state::{AuthState, AuthStateError, AuthStateManager, SignedState},
    tokens::{ResponseTokens, TokenClaims},
};

#[tracing::instrument(name = "authn::service::signin-callback", skip_all)]
pub async fn handle(
    ctx: &base::Ctx,
    client_info: &base::ReqClientInfo,
    code: &str,
    state: &str,
) -> Result<serde_json::Value, LoginError> {
    let callback_req = CallbackReq {
        dex: ctx.dex.clone(),
        client_ip: client_info.ip.clone(),
        client_user_agent: client_info.user_agent.clone(),
    };

    let callback_handler = CallbackHandler::new(&ctx.redis_pool);

    let (auth_state, _tokens, _claims) = callback_handler
        .handle(&callback_req, code, state)
        .await
        .map_err(|err| LoginError::CallbackHandler(err))?;

    // todo: get the session max lifetime and extension period from the org-config
    // let _session_max_lifetime_duration = std::time::Duration::from_secs(60 * 60 * 24 * 30);
    // let _session_max_lifetime = chrono::Duration::from_std(session_max_lifetime_duration).unwrap(); // 30 days

    Ok(serde_json::json!({"session-id": auth_state.user_session_id}))
}

/// Request for Handling the Callback
#[derive(Debug)]
pub struct CallbackReq {
    pub dex: base::ctx::settings::dex::DexSettings,
    pub client_ip: Option<String>,
    pub client_user_agent: Option<String>,
}

/// Service for handling the sign-in callback
pub struct CallbackHandler {
    state_manager: AuthStateManager,
}

impl CallbackHandler {
    pub fn new(redis_pool: &base::RedisPool) -> Self {
        Self {
            state_manager: AuthStateManager::new(redis_pool.clone()),
        }
    }

    // decode the state and verify the signature
    // get the auth-state from cache and veirfy the state params like expiry
    // exchange the code with the token with pkce-verifier
    // extract the details from the token-response
    #[tracing::instrument(name = "sign-in::callback::handler", skip_all)]
    pub async fn handle(
        &self,
        req: &CallbackReq,
        code: &str,
        state_id: &str,
    ) -> Result<(AuthState, ResponseTokens, TokenClaims), CallbackError> {
        tracing::info!(msg = "callback-request", ?code, ?state_id);

        // retrieve the signed state
        let signed_state = SignedState::decode(state_id, "todo:secret")
            .map_err(|err| CallbackError::SignedStateDecode(err))?;

        tracing::info!(
            msg = "decoded-signed-state",
            state_id = signed_state.state_id
        );

        // retrieve the auth-state
        let auth_state = self
            .state_manager
            .retrieve_auth_state(&signed_state.state_id)
            .await
            .map_err(|err| CallbackError::AuthStateRetrieve(err))?
            .ok_or_else(|| CallbackError::AuthStateNotFound)?;

        // validate the login-session-state
        if !self
            .state_manager
            .validate(&auth_state, &req.client_ip, &req.client_user_agent)
        {
            return Err(CallbackError::AuthStateValidation);
        }

        tracing::info!(msg = "attempting-token-exchange");

        let (token_response, id_token_claims) = self
            .exchanges_code_for_tokens(&req.dex, code, &auth_state.code_verifier, &auth_state.nonce)
            .await?;

        self.state_manager
            .invalidate_auth_state(&auth_state.user_session_id)
            .await
            .map_err(|err| CallbackError::AuthStateInvalidate(err))?;

        let claims_from_id_token = TokenClaims::from_id_token_claims(&id_token_claims);
        let response_tokens = ResponseTokens::from_token_response(&token_response);
        Ok((auth_state, response_tokens, claims_from_id_token))
    }

    #[tracing::instrument(name = "auth::callback::exchange-code", skip_all, err)]
    pub async fn exchanges_code_for_tokens(
        &self,
        dex_config: &DexSettings,
        code: &str,
        code_verifier: &str,
        expected_nonce: &str,
    ) -> Result<(CoreTokenResponse, CoreIdTokenClaims), CallbackError> {
        tracing::info!(msg = "token-exchange-request", code = code);

        // IDP Client: For Now we are using DEX IDP
        let issuer_url = IssuerUrl::new(dex_config.issuer_url.clone()).map_err(|error| {
            CallbackError::BuildIssueURLError(format!("issuer-url error: {}", error))
        })?;

        let http_client = HttpClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| CallbackError::BuildRequestClientError(error))?;

        // Fetch IDP metadata
        let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &http_client)
            .await
            .map_err(|error| {
                CallbackError::FetchProviderMetadataError(format!(
                    "idp provider metadata error: {}",
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
                        CallbackError::BuildRedirectURLError(format!(
                            "redirect-url error: {}",
                            error
                        ))
                    },
                )?);

        tracing::info!(msg = "making-token-exchange-request", code = code);
        // Exchange the authorization code for token with PKCE
        let token_response = client
            .exchange_code(AuthorizationCode::new(code.to_owned()))
            .map_err(|err| CallbackError::FailedToCreateAuthExchReq(err.to_string()))?
            .set_pkce_verifier(PkceCodeVerifier::new(code_verifier.to_string()))
            .request_async(&http_client)
            .await
            .map_err(|err| {
                tracing::error!(
                    msg = "token-exchange-failed",
                    error = %err,
                    code_verifier_prefix = &code_verifier[..8.min(code_verifier.len())]
                );
                CallbackError::FailedToExchTokenForCode(err.to_string())
            })?;

        tracing::info!(msg = "success-token-exchange", code = code);

        // Get ID Token
        let id_token = token_response
            .id_token()
            .ok_or_else(|| CallbackError::IDTokenExpectedInTokenResponse)?;

        // Verify ID token signature and claims using JWKS
        let id_token_verifier = client.id_token_verifier();
        let nonce_verifier = Nonce::new(expected_nonce.to_owned());

        // Verify the ID Token and Collect Claims
        let claims = id_token
            .claims(&id_token_verifier, &nonce_verifier)
            .map_err(|_err| CallbackError::IDTokenVerificationFailed)?
            .clone();

        tracing::info!(msg = "id-token-verified", code = code, ?claims);
        Ok((token_response, claims))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CallbackError {
    #[error("SignedStateDecodeError: {}", _0)]
    SignedStateDecode(AuthStateError),
    #[error("AuthStateNotFoundError")]
    AuthStateNotFound,
    #[error("SignedStateDecodeError: {}", _0)]
    AuthStateRetrieve(AuthStateError),
    #[error("AuthStateValidationError")]
    AuthStateValidation,
    #[error("BuildIssueURLError: {0}")]
    BuildIssueURLError(String),
    #[error("BuildRequestClientError: {0}")]
    BuildRequestClientError(#[from] reqwest::Error),
    #[error("FetchProviderMetadataError: {0}")]
    FetchProviderMetadataError(String),
    #[error("BuildRedirectURLError: {0}")]
    BuildRedirectURLError(String),
    #[error("FailedToCreateAuthExchReqError: {0}")]
    FailedToCreateAuthExchReq(String),
    #[error("FailedToExchTokenForCodeError: {0}")]
    FailedToExchTokenForCode(String),
    #[error("IDTokenExpectedInTokenResponse: Server did not return an ID Token")]
    IDTokenExpectedInTokenResponse,
    #[error("IDTokenVerificationFailed: Failed to verify ID token using nonce and token-verifier")]
    IDTokenVerificationFailed,
    #[error("AuthStateInvalidateError: {}", _0)]
    AuthStateInvalidate(AuthStateError),
}
