use std::sync::{Arc, Mutex};

use poem::http::StatusCode;
use poem::Error;
use poem::{get, handler, Route, Result, error::NotFoundError};
use poem::web::{Data, Path};
use serde_json::Value;

use crate::db::Db;
use crate::items::model::{Item, ItemCreateBody, ItemUpdateBody};
use crate::response::GenericResponse;

const ITEM_TABLE_NAME: &str = "item";

#[handler]
fn get_all_items(db: Data<&Arc<Mutex<Db>>>) -> Result<GenericResponse<Vec<Item>>> {
    let db_ref = db.lock().or_else(|_| Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)))?;
    let items = db_ref.find_all::<Item>(String::from(ITEM_TABLE_NAME)).unwrap_or(Vec::new());

    Ok(GenericResponse::<Vec<Item>>{
        message: None,
        status_code_u16: StatusCode::OK.as_u16(),
        data: Some(items)
    })
}

#[handler]
fn get_item_by_id(Path(id): Path<u32>, db: Data<&Arc<Mutex<Db>>>) -> Result<GenericResponse<Item>> {
    let db_ref = db.lock().or_else(|_| Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)))?;
    let item = db_ref.find_by_id::<Item>(String::from(ITEM_TABLE_NAME), id)
        .ok_or(NotFoundError)?;

    Ok(GenericResponse::<Item>{
        message: None,
        status_code_u16: StatusCode::OK.as_u16(),
        data: Some(item)
    })
}

#[handler]
fn create_item(payload: ItemCreateBody, db: Data<&Arc<Mutex<Db>>>) -> Result<GenericResponse<Item>> {
    let mut db_ref = db.lock().or_else(|_| Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)))?;
    let id = db_ref.get_increment_last_id(ITEM_TABLE_NAME.to_string()).unwrap().unwrap();
    let to_insert = Item::new(id, payload.name);
    let item = db_ref
        .insert_or_update(ITEM_TABLE_NAME.to_string(), id, to_insert)
        .unwrap()
        .unwrap();
        

    Ok(GenericResponse::<Item>{
        message: None,
        status_code_u16: StatusCode::CREATED.as_u16(),
        data: Some(item)
    })
}

#[handler]
fn put_item(Path(id): Path<u32>, payload: ItemUpdateBody, db: Data<&Arc<Mutex<Db>>>) -> Result<GenericResponse<Item>> {
    let mut db_ref = db.lock().or_else(|_| Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)))?;
    db_ref
        .find_by_id::<Item>(ITEM_TABLE_NAME.to_string(), id)
        .ok_or(NotFoundError)?;
    let to_update = Item::new(id, payload.name);
    db_ref
        .insert_or_update(ITEM_TABLE_NAME.to_string(), id, to_update.clone())
        .unwrap();

    Ok(GenericResponse::<Item>{
        message: None,
        status_code_u16: StatusCode::OK.as_u16(),
        data: Some(to_update)
    })
}

#[handler]
fn delete_item(Path(id): Path<u32>, db: Data<&Arc<Mutex<Db>>>) -> Result<GenericResponse<Value>> {
    let mut db_ref = db.lock().or_else(|_| Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)))?;
    db_ref
        .delete_by_id(ITEM_TABLE_NAME.to_string(), id)
        .unwrap();

    Ok(GenericResponse::<Value>{
        message: Some("Item deleted successfully".to_string()),
        status_code_u16: StatusCode::OK.as_u16(),
        data: None
    })
}


pub fn item_routes() -> Route {
    return Route::new()
        .at("/", get(get_all_items).post(create_item))
        .at(
            "/:id", 
            get(get_item_by_id).put(put_item).delete(delete_item)
        )
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use poem::{http::StatusCode, middleware::AddData, test::TestClient, EndpointExt};

    use crate::test::async_run_with_file_create_teardown;

    use super::*;

    fn insert_item(db: &mut Db, name: String) {
        let table_name = "item".to_string();
        db.add_table(table_name.clone(), false).unwrap();
        let id = db.get_increment_last_id(table_name.clone()).unwrap().unwrap();
        let to_insert = Item::new(id, name);
        db.insert_or_update(table_name.clone(), id, to_insert).unwrap();
    }

    #[tokio::test]
    async fn test_get_all_items() {
        async_run_with_file_create_teardown(|| async {
            let routes = item_routes();
            let mut db = Db::init(String::from("./data.json")).unwrap();
            db.delete_all("item".to_string()).unwrap();
    
            insert_item(&mut db, String::from("item 1"));
            insert_item(&mut db, String::from("item 2"));
            insert_item(&mut db, String::from("item 3"));
    
            let arc_db = Arc::new(Mutex::new(db));
    
            let client = TestClient::new(
                Route::new().nest("/items", routes)
                    .with(AddData::new(arc_db))
            );
            let response = client.get("/items").send().await;
    
            let expected_data = serde_json::json!({
                "data": [
                    {
                        "id": 1,
                        "name": "item 1"
                    },
                    {
                        "id": 2,
                        "name": "item 2"
                    },
                    {
                        "id": 3,
                        "name": "item 3"
                    }
                ]
            });
    
            response.assert_status_is_ok();
            response.assert_json(expected_data).await;
        }).await;
    }

    #[tokio::test]
    async fn test_get_item_by_id() {
        async_run_with_file_create_teardown(|| async {
            let routes = item_routes();
            let mut db = Db::init(String::from("./data.json")).unwrap();
            db.delete_all("item".to_string()).unwrap();
    
            insert_item(&mut db, String::from("item 1"));
            insert_item(&mut db, String::from("item 2"));
            insert_item(&mut db, String::from("item 3"));
    
            let arc_db = Arc::new(Mutex::new(db));
    
            let client = TestClient::new(
                Route::new().nest("/items", routes)
                    .with(AddData::new(arc_db))
            );
    
            let response = client.get("/items/2").send().await;
    
            let expected_data = serde_json::json!({
                "data": {
                    "id": 2,
                    "name": "item 2"
                }
            });
    
            response.assert_status_is_ok();
            response.assert_json(expected_data).await;
        }).await;
    }

    #[tokio::test]
    async fn test_get_item_by_id_not_found() {
        async_run_with_file_create_teardown(|| async {
            let routes = item_routes();
            let mut db = Db::init(String::from("./data.json")).unwrap();
            db.delete_all("item".to_string()).unwrap();
    
            let arc_db = Arc::new(Mutex::new(db));
    
            let client = TestClient::new(
                Route::new().nest("/items", routes)
                    .with(AddData::new(arc_db))
            );
    
            let response = client.get("/items/2").send().await;
    
            response.assert_status(StatusCode::NOT_FOUND);
        }).await;
    }

    #[tokio::test]
    async fn test_create_item() {
        async_run_with_file_create_teardown(|| async {
            let routes = item_routes();
            let mut db = Db::init(String::from("./data.json")).unwrap();
            db.add_table("item".to_string(), false).unwrap();
            db.delete_all("item".to_string()).unwrap();
    
            let arc_db = Arc::new(Mutex::new(db));
    
            let client = TestClient::new(
                Route::new().nest("/items", routes)
                    .with(AddData::new(arc_db))
            );
    
            let response = client.post("/items")
                .body_json(&ItemCreateBody{ name: "item 1".to_string() })
                .send()
                .await;
    
            let expected_data = serde_json::json!({
                "data": {
                    "id": 1,
                    "name": "item 1"
                }
            });
    
            response.assert_status(StatusCode::CREATED);
            response.assert_json(expected_data).await;
        }).await;
    }

    #[tokio::test]
    async fn test_put_item() {
        async_run_with_file_create_teardown(|| async {

            let routes = item_routes();
            let mut db = Db::init(String::from("./data.json")).unwrap();
            db.add_table("item".to_string(), false).unwrap();
            db.delete_all("item".to_string()).unwrap();
    
            insert_item(&mut db, "item 1".to_string());
    
            let arc_db = Arc::new(Mutex::new(db));
    
            let client = TestClient::new(
                Route::new().nest("/items", routes)
                    .with(AddData::new(arc_db))
            );
    
            let put_response = client.put("/items/1")
                .body_json(&ItemUpdateBody{ name: "item 1 updated".to_string() })
                .send()
                .await;
    
            let get_response = client.get("/items/1")
                .send()
                .await;
            
            put_response.assert_status_is_ok();
            get_response.assert_status_is_ok();
            get_response.assert_json(put_response.json().await).await;
        }).await;
    }

    #[tokio::test]
    async fn test_delete_item() {
        async_run_with_file_create_teardown(|| async {
            let routes = item_routes();
            let mut db = Db::init(String::from("./data.json")).unwrap();
            db.add_table("item".to_string(), false).unwrap();
            db.delete_all("item".to_string()).unwrap();
    
            insert_item(&mut db, "item 1".to_string());
    
            let arc_db = Arc::new(Mutex::new(db));
    
            let client = TestClient::new(
                Route::new().nest("/items", routes)
                    .with(AddData::new(arc_db))
            );
    
            let response = client.delete("/items/1")
                .send()
                .await;

            response.assert_status_is_ok();
        }).await;
    }
}