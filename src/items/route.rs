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
    let db_ref = db
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
        .expect("Getting db lock");
    let items = db_ref
        .find_all::<Item>(String::from(ITEM_TABLE_NAME))
        .unwrap_or_default();

    Ok(GenericResponse::<Vec<Item>>{
        message: None,
        status_code_u16: StatusCode::OK.as_u16(),
        data: Some(items)
    })
}

#[handler]
fn get_item_by_id(Path(id): Path<u32>, db: Data<&Arc<Mutex<Db>>>) -> Result<GenericResponse<Item>> {
    let db_ref = db
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
        .expect("Getting db lock");
    let item = db_ref.find_by_id::<Item>(String::from(ITEM_TABLE_NAME), id)
        .ok_or(NotFoundError)?;

    Ok(GenericResponse::<Item>{
        message: None,
        status_code_u16: StatusCode::OK.as_u16(),
        data: Some(item)
    })
}

#[poem_grants::protect("MUTATE")]
#[handler]
fn create_item(payload: ItemCreateBody, db: Data<&Arc<Mutex<Db>>>) -> Result<GenericResponse<Item>> {
    
    let mut db_ref = db
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
        .expect("Getting db lock");
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

#[poem_grants::protect("MUTATE")]
#[handler]
fn put_item(Path(id): Path<u32>, payload: ItemUpdateBody, db: Data<&Arc<Mutex<Db>>>) -> Result<GenericResponse<Item>> {
    let mut db_ref = db
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
        .expect("Getting db lock");
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

#[poem_grants::protect("MUTATE")]
#[handler]
fn delete_item(Path(id): Path<u32>, db: Data<&Arc<Mutex<Db>>>) -> Result<GenericResponse<Value>> {
    let mut db_ref = db
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
        .expect("Getting db lock");
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
    Route::new()
        .at("/", get(get_all_items).post(create_item))
        .at(
            "/:id", 
            get(get_item_by_id).put(put_item).delete(delete_item)
        )
}

#[cfg(test)]
mod tests {
    use poem::{http::StatusCode, Endpoint};

    use crate::test::{async_run_with_file_create_teardown, ApiTestClient};

    use super::*;

    fn insert_item(db: &mut Db, name: String) {
        let table_name = "item".to_string();
        db.add_table(table_name.clone(), false).unwrap();
        let id = db.get_increment_last_id(table_name.clone()).unwrap().unwrap();
        let to_insert = Item::new(id, name);
        db.insert_or_update(table_name.clone(), id, to_insert).unwrap();
    }

    fn init_client() -> ApiTestClient<impl Endpoint> {
        let routes = Route::new().nest(
            "/items", item_routes()
        );
        let test_client = ApiTestClient::init(routes);
        {
            let mut db = test_client.db.lock().unwrap();
            db.add_table("item".to_string(), false).unwrap();
            db.delete_all("item".to_string()).unwrap();
        }

        return test_client
    }

    #[tokio::test]
    async fn test_get_all_items() {
        async_run_with_file_create_teardown(|| async {
            let test_client = init_client();
            {
                let mut db = test_client.db.lock().unwrap();
                insert_item(&mut db, String::from("item 1"));
                insert_item(&mut db, String::from("item 2"));
                insert_item(&mut db, String::from("item 3"));
            }
            let response = test_client.client.get("/items").send().await;
    
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
            let test_client = init_client();
            {
                let mut db = test_client.db.lock().unwrap();
                insert_item(&mut db, String::from("item 1"));
                insert_item(&mut db, String::from("item 2"));
                insert_item(&mut db, String::from("item 3"));
            }
    
            let response = test_client.client.get("/items/2").send().await;
    
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
            let test_client = init_client();
            let response = test_client.client.get("/items/99").send().await;
    
            response.assert_status(StatusCode::NOT_FOUND);
        }).await;
    }

    #[tokio::test]
    async fn test_create_item() {
        async_run_with_file_create_teardown(|| async {      
            let test_client = init_client();
    
            let response = test_client.client.post("/items")
                .body_json(&ItemCreateBody{ name: "item 1".to_string() })
                .header("Authorization", format!("Bearer {}", test_client.token))
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
            let test_client = init_client();

            {
                let mut db = test_client.db.lock().unwrap();
                insert_item(&mut db, "item 1".to_string());
            }
    
            let put_response = test_client.client.put("/items/1")
                .body_json(&ItemUpdateBody{ name: "item 1 updated".to_string() })
                .header("Authorization", format!("Bearer {}", test_client.token))
                .send()
                .await;
    
            let get_response = test_client.client.get("/items/1")
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
            let test_client = init_client();

            let response = test_client.client.delete("/items/1")
                .header("Authorization", format!("Bearer {}", test_client.token))
                .send()
                .await;

            response.assert_status_is_ok();
        }).await;
    }
}