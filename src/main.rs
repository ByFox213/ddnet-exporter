mod models;
mod register;
mod util;

use std::net::SocketAddr;

use crate::models::Config;
use crate::util::ddnet;
use hyper::Request;
use hyper::Response;
use hyper::body::Incoming;
use hyper::header::CONTENT_TYPE;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use log::{error, info};
use prometheus::{Encoder, TextEncoder};
use tokio::net::TcpListener;

type BoxedErr = Box<dyn std::error::Error + Send + Sync + 'static>;

async fn serve_req(_req: Request<Incoming>) -> Result<Response<String>, BoxedErr> {
    let encoder = TextEncoder::new();

    let metric_families = prometheus::gather();
    let body = encoder.encode_to_string(&metric_families)?;

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(body)?;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), BoxedErr> {
    let config = Config::from_env().expect("Failed loading config from env");
    config.set_logging();
    let addr: SocketAddr = ([0, 0, 0, 0], config.web_port).into();
    info!("Listening on http://127.0.0.1:{}", config.web_port);
    let listener = TcpListener::bind(addr).await?;
    tokio::spawn(ddnet(config));

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let service = service_fn(serve_req);
        if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
            error!("web: server error: {err:?}");
        };
    }
}
