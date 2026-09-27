use anyhow::Result;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use rust_embed::RustEmbed;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(RustEmbed)]
#[folder = "web/"]
struct WebAssets;

pub async fn run_server(token: String) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("HTTP: http://{}", addr);
    let listener = TcpListener::bind(addr).await?;
    let token = Arc::new(token);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let token = token.clone();

        tokio::task::spawn(async move {
            let svc = service_fn(move |req| {
                let token = token.clone();
                async move { handle_request(req, token).await }
            });
            if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                tracing::debug!("connection error: {}", e);
            }
        });
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    token: Arc<String>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path();
    let path = if path == "/" {
        "index.html"
    } else {
        &path[1..]
    };

    if path == "api/token" {
        return Ok(Response::builder()
            .header("Content-Type", "application/json")
            .header("X-Content-Type-Options", "nosniff")
            .body(Full::new(Bytes::from(format!(
                "{{\"token\":\"{}\"}}",
                token
            ))))
            .unwrap());
    }

    match WebAssets::get(path) {
        Some(c) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Ok(Response::builder()
                .header("Content-Type", mime.as_ref())
                .header("X-Content-Type-Options", "nosniff")
                .header("Referrer-Policy", "no-referrer")
                .body(Full::new(Bytes::from(c.data.to_vec())))
                .unwrap())
        }
        None => match WebAssets::get("index.html") {
            Some(c) => Ok(Response::builder()
                .header("Content-Type", "text/html")
                .body(Full::new(Bytes::from(c.data.to_vec())))
                .unwrap()),
            None => Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("404")))
                .unwrap()),
        },
    }
}
