pub mod db;
pub mod items;
pub mod test;
pub mod response;
pub mod auth;

use std::sync::{Arc, Mutex};

use auth::route::auth_routes;
use poem::middleware::{AddData, Tracing};
use poem::Middleware;
use poem::{listener::TcpListener, EndpointExt, Route, Server};
use response::GenericResponse;
use serde_json::Value;

use crate::items::route::item_routes;
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

    let jwt_manager = auth::jwt::Manager::init("secret".to_string(), 24);
    let jwt_middleware = auth::middleware::JwtMiddleware{ manager: jwt_manager.clone() };
    
    let app = Route::new()
        .nest("/items", item_routes())
        .nest("/", auth_routes())
        .with(
            jwt_middleware
                .combine(AddData::new(db_ref))
                .combine(AddData::new(jwt_manager))
                .combine(Tracing)
        )
        .catch_all_error(|err| async move {
            GenericResponse::<Value>{ 
                message: Some(err.to_string()),
                status_code_u16: err.status().as_u16(),
                data: None
            }
        });
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}