pub struct HttpService;

impl hyper::service::Service<hyper::Request<hyper::Body>> for HttpService {
    type Response = hyper::Response<hyper::Body>;
    type Error = hyper::Error;
    type Future = std::pin::Pin<
        Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: hyper::Request<hyper::Body>) -> Self::Future {
        Box::pin(async {
            match service::router::handler(req).await {
                Ok(r) => Ok(r),
                Err(_e) => {
                    dbg!(_e);
                    unimplemented!()
                }
            }
        })
    }
}

fn env() -> String {
    match std::env::var("ENV") {
        Ok(env) => env.to_lowercase(),
        Err(_) => "local".to_string(),
    }
}

async fn http_main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    // Setting the environment variables
    let env_path = format!("{}.env", env());
    dotenv::from_path(env_path.as_str()).ok();
    println!("Environment set: {}", env_path);

    // Initializing the database pool
    db::pg::init_db_pool();
    db::redis::init_redis_pool();

    // Creating the tcp listener
    let socket_address: std::net::SocketAddr = ([0, 0, 0, 0], 8000).into();
    let listener = tokio::net::TcpListener::bind(socket_address).await?;
    println!(
        "#### Started at: {}:{} ####",
        socket_address.ip(),
        socket_address.port()
    );
    loop {
        let (tcp_stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(http_err) = hyper::server::conn::Http::new()
                .serve_connection(tcp_stream, HttpService {})
                .await
            {
                eprintln!("Error while serving HTTP connection: {}", http_err);
            }
        });
    }
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(http_main())
        .unwrap();
}
