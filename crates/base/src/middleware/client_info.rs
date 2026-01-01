#[derive(Clone, Debug)]
pub struct ReqClientInfo {
    // Note: Service must behind the well defined proxies and set the header x-forwarded-for
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub host: (Option<String>, Option<String>),
    pub uri: axum::http::Uri,
    pub referrer: Option<String>,
    pub scheme: Option<String>,
    pub accept_language: Option<String>,
    // Mostly set by the proxies, Note: We should not use scheme which is get by the URI
    pub forwarded_proto: Option<String>,
}
