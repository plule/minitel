use std::net::SocketAddr;

use crate::app::App;
use futures::{AsyncRead, AsyncWrite};
use tokio_util::compat::TokioAsyncReadCompatExt;
use tracing::Level;

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let address = std::env::args()
        .nth(1)
        .unwrap_or("127.0.0.1:3615".to_string());
    log::info!("Listening on {}", address);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    loop {
        if let Ok((stream, socket)) = listener.accept().await {
            log::info!("Accepted connection from {}", socket);
            tokio::spawn(async move {
                serve(stream.compat(), socket).await;
            });
        }
    }
}

pub async fn serve<T: AsyncWrite + AsyncRead + Unpin>(mut stream: T, socket: SocketAddr) {
    log::info!("Serving {}", socket);
    let r = App::default().run(&mut stream).await;
    match r {
        Ok(_) => log::info!("Connection with {} closed", socket),
        Err(e) => log::error!("Connection with {} closed with error: {:?}", socket, e),
    }
}
