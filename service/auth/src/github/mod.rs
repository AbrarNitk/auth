pub const CALLBACK_URL: &str = "/auth/github/callback/";

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
    req: &hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, ()> {
    // TODO: add the next url as query parameter and platform is github

    let host = req.headers().get(hyper::header::HOST).map(|x| x.to_str().unwrap().to_string()).unwrap();
    println!("host: {}, uri: {:?}", host, req.uri());

    let scheme = match req.uri().scheme() {
        Some(scheme) => scheme.to_string(),
        None => "http".to_string()
    };

    println!("scheme {:?}", scheme);
    let callback_url = format!("{}://{}{}", scheme, host, CALLBACK_URL);
    println!("callback url {}", callback_url);
    let client = client().set_redirect_uri(oauth2::RedirectUrl::new(callback_url).unwrap());

    let (mut authorize_url, _token) = client
        // this is the state that we are passing, and will receive in the callback url
        // todo: we must store it some where to validate the request, but for that we will need
        // user tacker to implement
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .add_scope(oauth2::Scope::new("read:user".to_string()))
        .add_scope(oauth2::Scope::new("read:org".to_string()))
        .add_scope(oauth2::Scope::new("public_repo".to_string()))
        .url();

    // Note: asking the consent from user
    authorize_url
        .query_pairs_mut()
        .append_pair("prompt", "consent");


    // Note: setting response with redirect url as authorize url
    let mut resp = hyper::Response::new(hyper::Body::empty());
    let location = hyper::header::HeaderValue::from_bytes(authorize_url.as_str().as_bytes())
        .expect("something went wrong");
    resp.headers_mut().insert(hyper::header::LOCATION, location);
    *resp.status_mut() = hyper::StatusCode::PERMANENT_REDIRECT;
    Ok(resp)
}

// TODO: remove the unwraps
pub(crate) async fn callback(req: &hyper::Request<hyper::Body>)-> Result<hyper::Response<hyper::Body>, ()>  {
    use oauth2::TokenResponse;
    let host = req.headers().get(hyper::header::HOST).map(|x| x.to_str().unwrap().to_string()).unwrap();
    let scheme = match req.uri().scheme() {
        Some(scheme) => scheme.to_string(),
        None => "http".to_string()
    };
    println!("scheme: {}, host: {:?}", scheme, host);
    let query = url::form_urlencoded::parse(req.uri().query().unwrap().as_bytes())
        .into_owned()
        .collect::<std::collections::HashMap<String, String>>();
    println!("callback params: {:?}", query);
    let code = query.get("code").unwrap();
    let auth_url = format!("{}://{}{}", scheme, host, CALLBACK_URL);
    let client = client().set_redirect_uri(oauth2::RedirectUrl::new(auth_url).unwrap());
    match client.exchange_code(oauth2::AuthorizationCode::new(code.to_owned()))
        .request_async(oauth2::reqwest::async_http_client).await {
        Ok(token) => {
            let t =  token.access_token().secret();
            println!("get the token: {}", t);
            let cookie_value = format!("gt={}; HttpOnly; Path=/; Domain={}", t, sanitize_port(host.as_str()));
            println!("cookie: {}", cookie_value);
            let mut response = hyper::Response::builder()
                .status(hyper::StatusCode::PERMANENT_REDIRECT)
                .body(hyper::Body::empty())
                .unwrap();

            let cookie_header = hyper::header::HeaderValue::from_str(&cookie_value).expect("failed to create the cookie header");
            response.headers_mut().insert(hyper::header::SET_COOKIE, cookie_header);
            response.headers_mut().insert(hyper::header::LOCATION, hyper::header::HeaderValue::from_str("/").unwrap());
            return Ok(response)
            // creating the cookies to set in the response
        }
        Err(e) => {
            println!("token request error: {}", e);
            let mut resp = hyper::Response::new(hyper::Body::empty());
            *resp.status_mut() = hyper::StatusCode::PERMANENT_REDIRECT;
            return Ok(resp);
        }
    }

    // todo: set the callback url as redirect url, or else redirect it to the home page
    // todo: check the state same as we send in redirect uri as query param
    // todo: check the scope we asked for all the
    // we will get the code here send call to the github in exchange with te access_token
}

fn sanitize_port(host: &str) -> String {
    match host.split_once(":") {
        Some((domain, _port)) => domain.to_string(),
        None => host.to_string()
    }
}

#[derive(serde::Deserialize)]
pub struct QParams {
    code: String,
    state: String,
    next: Option<String>
}
