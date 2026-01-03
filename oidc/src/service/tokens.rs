use openidconnect::{
    OAuth2TokenResponse as _, TokenResponse as _,
    core::{CoreIdTokenClaims, CoreTokenResponse},
};

#[derive(Debug)]
pub struct TokenClaims {
    /// Stable Identifier for the end-user at the issuer
    pub subject: String,
    /// User's Email address, Note: Required Scope as email
    pub user_email: Option<String>,
    pub email_verified: Option<bool>,
    /// Full display name of the user
    pub username: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub middle_name: Option<String>,
    pub birth_date: Option<String>,
    /// User's Phone Number
    pub phone: Option<String>,
    pub phone_verified: Option<bool>,
    /// User's human friendly username
    pub preferred_username: Option<String>,
    /// User's profile URL
    pub profile: Option<String>,
    /// URL to the user’s profile image
    pub picture: Option<String>,

    pub issue_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// When the user last actively authenticated
    pub last_auth_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Authentication Context Class Reference
    pub auth_context_ref: Option<String>,

    /// Authentication Method references
    pub auth_method_refs: Option<Vec<String>>,
    /// Identifier URL of the OpenID provider that issued the token
    pub issuer: String,
    /// Recipients of the token
    pub audiences: Vec<String>,
    pub nonce: Option<String>,
}

impl TokenClaims {
    pub fn from_id_token_claims(claims: &CoreIdTokenClaims) -> Self {
        let user_locale = claims.locale();
        let email = claims.email().map(|n| n.as_str().to_string());
        let email_verified = claims.email_verified();
        let subject = claims.subject().as_str().to_string();
        let nonce = claims.nonce().map(|n| n.secret().to_string());
        let username = claims
            .name()
            .and_then(|n| n.get(user_locale))
            .map(|n| n.as_str().to_string());
        let preferred_username = claims.preferred_username().map(|n| n.as_str().to_string());
        let given_name = claims
            .given_name()
            .and_then(|n| n.get(user_locale))
            .map(|n| n.as_str().to_string());
        let family_name = claims
            .given_name()
            .and_then(|n| n.get(user_locale))
            .map(|n| n.as_str().to_string());
        let middle_name = claims
            .middle_name()
            .and_then(|n| n.get(user_locale))
            .map(|n| n.as_str().to_string());
        let birth_date = claims.birthdate().map(|dob| dob.as_str().to_string());
        let phone = claims.phone_number().map(|p| p.as_str().to_string());
        let phone_verified = claims.phone_number_verified();
        let profile = claims
            .profile()
            .and_then(|p| p.get(user_locale))
            .map(|p| p.as_str().to_string());
        let picture = claims
            .picture()
            .and_then(|p| p.get(user_locale))
            .map(|p| p.as_str().to_string());

        let issue_at = claims.issue_time();
        let expires_at = claims.expiration();
        // When the user last actively authenticated
        let auth_time = claims.auth_time();

        // Authnetication Context Reference
        let auth_context_ref = claims
            .auth_context_ref()
            .map(|acr| acr.as_str().to_string());

        // Authnetication Methods References
        let auth_method_refs = claims.auth_method_refs().map(|amr| {
            amr.iter()
                .map(|amr| amr.as_str().to_string())
                .collect::<Vec<_>>()
        });

        let issuer = claims.issuer().as_str().to_string();
        let audiences = claims
            .audiences()
            .iter()
            .map(|aud| aud.as_str().to_string())
            .collect::<Vec<_>>();

        Self {
            subject,
            user_email: email,
            email_verified,
            username,
            given_name,
            family_name,
            middle_name,
            birth_date,
            phone,
            phone_verified,
            preferred_username,
            profile,
            picture,
            issue_at,
            expires_at,
            last_auth_time: auth_time,
            auth_context_ref,
            auth_method_refs,
            issuer,
            audiences,
            nonce,
        }
    }
}

#[derive(Debug)]
pub struct ResponseTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub id_token: Option<String>,
    pub scopes: Vec<String>,
    pub token_type: String,
}

impl ResponseTokens {
    pub fn from_token_response(token_response: &CoreTokenResponse) -> Self {
        let access_token = token_response.access_token().secret().clone();
        let refresh_token = token_response.refresh_token().map(|rt| rt.secret().clone());
        let id_token = token_response.id_token().map(|t| t.to_string());
        let token_expires_at = token_response
            .expires_in()
            .map(|exp_secs| chrono::Utc::now() + exp_secs);
        let scopes = token_response
            .scopes()
            .map(|scopes| {
                scopes
                    .iter()
                    .map(|s| s.as_str().to_owned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let token_type = token_response.token_type().as_ref().to_owned();
        Self {
            access_token,
            refresh_token,
            expires_at: token_expires_at,
            id_token,
            scopes,
            token_type,
        }
    }
}
