use axum::{http::Request, response::Response};
use tower::{Layer, Service};

#[derive(Debug, Clone, Copy)]
pub struct ReqClientInfoLayer;

impl<S> Layer<S> for ReqClientInfoLayer {
    type Service = ReqClientInfoMiddlewareService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ReqClientInfoMiddlewareService { inner }
    }
}

#[derive(Debug, Clone)]
pub struct ReqClientInfoMiddlewareService<S> {
    inner: S,
}

impl<S, ReqBody> Service<Request<ReqBody>> for ReqClientInfoMiddlewareService<S>
where
    S: Service<Request<ReqBody>, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let header_map = req.headers();

        // Extact the IP
        let ip = client_ip::x_real_ip(header_map)
            .or_else(|_| client_ip::rightmost_x_forwarded_for(header_map))
            .map(|ip| ip.to_string())
            .ok();

        // User-Agent
        let user_agent = header_map
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|x| x.to_string());
        // Host and Port
        let (host, port) = match header_map.get("host").and_then(|v| v.to_str().ok()) {
            Some(h) => {
                let mut host_parts = h.splitn(2, ":");
                let host = host_parts.next().map(|x| x.to_string());
                let port = host_parts.next().map(|x| x.to_string());
                (host, port)
            }
            None => (None, None),
        };

        // Referrer
        let referrer = header_map
            .get("referrer")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let accept_language = header_map
            .get("accept-language")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Forwarded Proto
        let forwarded_proto = header_map
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Scheme inference: prefer x-forwarded-proto, then URI scheme as fallback
        let scheme = forwarded_proto
            .clone()
            .or_else(|| req.uri().scheme_str().map(|s| s.to_string()));

        let client_info = base::ReqClientInfo {
            ip,
            user_agent,
            host: (host, port),
            referrer,
            scheme,
            accept_language,
            forwarded_proto,
            uri: req.uri().clone(),
        };

        req.extensions_mut().insert(client_info);
        self.inner.call(req)
    }
}
