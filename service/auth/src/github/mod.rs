pub const CALLBACK_URL: &str = "/auth/github/callback/?platform=github";

static CLIENT_ID: once_cell::sync::Lazy<oauth2::ClientId> = {
    once_cell::sync::Lazy::new(|| {
        oauth2::ClientId::new(match std::env::var("GITHUB_CLIENT_ID") {
            Ok(val) => val,
            Err(e) => panic!("{}{}", "GITHUB_CLIENT_ID not found in env ", e),
        })
    })
};

static CLIENT_SECRET: once_cell::sync::Lazy<oauth2::ClientSecret> = {
    once_cell::sync::Lazy::new(|| {
        oauth2::ClientSecret::new(match std::env::var("GITHUB_CLIENT_SECRET") {
            Ok(val) => val,
            Err(e) => panic!("{}{}", "GITHUB_CLIENT_SECRET not found in env ", e),
        })
    })
};

pub(crate) fn client() -> oauth2::basic::BasicClient {
    oauth2::basic::BasicClient::new(
        CLIENT_ID.to_owned(),
        Some(CLIENT_SECRET.to_owned()),
        oauth2::AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
        Some(
            oauth2::TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                .expect("Invalid token endpoint URL"),
        ),
    )
}


pub(crate) async fn login(
    _req: &hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, ()> {

    // TODO: add the next url as query parameter and platform is github
    let callback_url = format!("http://127.0.0.1:8001{}", CALLBACK_URL);
    let client = client().set_redirect_uri(oauth2::RedirectUrl::new(callback_url).unwrap());

    let (mut authorize_url, _token) = client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .add_scope(oauth2::Scope::new("read:user".to_string()))
        .add_scope(oauth2::Scope::new("read:org".to_string()))
        .add_scope(oauth2::Scope::new("public_repo".to_string()))
        .url();

    authorize_url
        .query_pairs_mut()
        .append_pair("prompt", "consent");

    let mut resp = hyper::Response::new(hyper::Body::empty());
    let location = hyper::header::HeaderValue::from_bytes(authorize_url.as_str().as_bytes())
        .expect("something went wrong");
    resp.headers_mut().insert(hyper::header::LOCATION, location);
    *resp.status_mut() = hyper::StatusCode::PERMANENT_REDIRECT;
    Ok(resp)
}

pub(crate) async fn callback(_req: &hyper::Request<hyper::Body>)-> Result<hyper::Response<hyper::Body>, ()>  {
    // todo: set the callback url
    // let query = url::form_urlencoded::parse(req.uri().query().unwrap_or("").as_bytes())
    //     .into_owned()
    //     .collect::<std::collections::HashMap<String, String>>();
    // we will get the code here send call to the github in exchange with te access_token
    let mut resp = hyper::Response::new(hyper::Body::empty());
    *resp.status_mut() = hyper::StatusCode::PERMANENT_REDIRECT;
    Ok(resp)
}

