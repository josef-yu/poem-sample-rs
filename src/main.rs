pub mod db;
pub mod items;
pub mod test;
pub mod response;
pub mod auth;

use std::sync::{Arc, Mutex};

use auth::route::AuthApi;
use clap::Parser;
use items::route::ItemsApi;
use poem::middleware::{AddData, Tracing};
use poem::Middleware;
use poem::{listener::TcpListener, EndpointExt, Route, Server};
use poem_openapi::OpenApiService;

use crate::db::Db;

#[derive(Debug, Parser)]
#[clap(author, version, about, disable_help_flag = true)]
pub struct ServerConfig {
    #[clap(short, long, env = "HOST", default_value = "0.0.0.0")]
    /// The binding host address of the server.
    pub host: String,

    #[clap(short, long, env = "PORT", default_value = "3000")]
    pub port: u16,

    #[clap(short = 's', long, env = "JWT_SECRET", default_value = "secret")]
    pub jwt_secret: String,

    #[clap(short = 'd', long, env = "JWT_HOUR_DURATION", default_value = "24")]
    pub jwt_hour_duration: i64,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = ServerConfig::parse();
    let address = format!("{}:{}", config.host, config.port);

    tracing_subscriber::fmt()
        .with_env_filter("poem=trace")
        .init();

    let mut db = Db::init("./data.json".to_string()).expect("Initializing db");
    db.add_table("item".to_string(), false).unwrap();
    db.add_table("user".to_string(), false).unwrap();
    let db_ref = Arc::new(Mutex::new(db));

    let api_service = OpenApiService::new((AuthApi, ItemsApi), "Poem api", "1");

    let jwt_manager = auth::jwt::Manager::init(config.jwt_secret, config.jwt_hour_duration);
    let jwt_middleware = auth::middleware::JwtMiddleware{ manager: jwt_manager.clone() };
    
    let app = Route::new()
        .nest("", api_service)
        .with(
            jwt_middleware
                .combine(AddData::new(db_ref))
                .combine(AddData::new(jwt_manager))
                .combine(Tracing)
        );
    Server::new(TcpListener::bind(address))
        .run(app)
        .await
}