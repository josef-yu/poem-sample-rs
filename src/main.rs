pub mod db;
pub mod items;
pub mod test;
pub mod response;
pub mod auth;

use std::sync::{Arc, Mutex};

use auth::route::AuthApi;
use items::route::ItemsApi;
use poem::middleware::{AddData, Tracing};
use poem::Middleware;
use poem::{listener::TcpListener, EndpointExt, Route, Server};
use poem_openapi::OpenApiService;

use crate::db::Db;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt()
        .with_env_filter("poem=trace")
        .init();

    let mut db = Db::init("./data.json".to_string()).expect("Initializing db");
    db.add_table("item".to_string(), false).unwrap();
    db.add_table("user".to_string(), false).unwrap();
    let db_ref = Arc::new(Mutex::new(db));

    let api_service = OpenApiService::new((AuthApi, ItemsApi), "Poem api", "1");

    let jwt_manager = auth::jwt::Manager::init("secret".to_string(), 24);
    let jwt_middleware = auth::middleware::JwtMiddleware{ manager: jwt_manager.clone() };
    
    let app = Route::new()
        .nest("", api_service)
        .with(
            jwt_middleware
                .combine(AddData::new(db_ref))
                .combine(AddData::new(jwt_manager))
                .combine(Tracing)
        );
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}