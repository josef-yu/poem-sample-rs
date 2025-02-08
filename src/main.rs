pub mod db;
pub mod items;
pub mod app;
pub mod test;
pub mod response;

use std::sync::{Arc, Mutex};

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

    let mut db = Db::init("./data.json".to_string()).expect("Failed to initialize db");
    db.add_table("item".to_string(), false).unwrap();

    let db_ref = Arc::new(Mutex::new(db));

    let app = Route::new()
        .nest("/items", item_routes())
        .with(AddData::new(db_ref).combine(Tracing))
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