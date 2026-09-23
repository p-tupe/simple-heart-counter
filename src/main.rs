use axum::{
    Router,
    routing::{get, post},
};
use axum_client_ip::ClientIpSource;
use std::{env, error::Error, fs, net::SocketAddr};
use tokio_rusqlite::Connection;
use tower_http::cors::Any;

use crate::handlers::{AppState, decrement_count, get_count, increment_count, initialize};

mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let script = fs::read_to_string("shc.js").expect("read shc.js");
    let db = Connection::open("./shc.db").await?;

    initialize(&db).await?;

    let app = Router::new()
        .route("/shc.js", get(script))
        .route("/count", post(get_count))
        .route("/count/increment", post(increment_count))
        .route("/count/decrement", post(decrement_count))
        .layer(ClientIpSource::ConnectInfo.into_extension())
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(AppState { db });

    let port = &env::var("PORT").unwrap_or("3000".into());
    let addr = format!("127.0.0.1:{}", port);
    log::info!("starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok(axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?)
}
