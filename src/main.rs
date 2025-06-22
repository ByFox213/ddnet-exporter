mod models;
mod register;
mod util;

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
use std::net::SocketAddr;
use tokio::net::TcpListener;

async fn serve_req(_req: Request<Incoming>) -> anyhow::Result<Response<String>> {
    let encoder = TextEncoder::new();

    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer)?;
    let body = String::from_utf8(buffer)?;

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(body)?;

    Ok(response)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env().expect("Failed loading config from env");
    config.set_logging();

    let addr: SocketAddr = ([0, 0, 0, 0], config.web_port).into();
    let listener = TcpListener::bind(addr).await.expect("Failed bind to add");

    info!(
        "Listening on http://0.0.0.0:{0}, fast url: http://127.0.0.1:{0}",
        config.web_port
    );
    tokio::spawn(ddnet(config));

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("New connection from {addr}");

                let io = TokioIo::new(stream);
                let service = service_fn(serve_req);

                let connection = http1::Builder::new()
                    .serve_connection(io, service)
                    .with_upgrades();

                if let Err(err) = connection.await {
                    error!("web: server error: {err:?}");
                };
            }
            Err(err) => error!("Accept connection error: {err:?}"),
        }
    }
}
