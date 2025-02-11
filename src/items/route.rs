use std::sync::{Arc, Mutex};

use poem::Result;
use poem::web::{Data, Path};
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;

use crate::db::Db;
use crate::items::model::{Item, ItemCreateBody, ItemUpdateBody, ItemNotFound};
use crate::response::{CreateResponse, DeleteResponse, FetchResponse, GenericError, UpdateResponse};

use super::model::ItemDelete;

const ITEM_TABLE_NAME: &str = "item";

pub struct ItemsApi;

#[poem_grants::open_api]
#[OpenApi(prefix_path = "/items")]
impl ItemsApi {

    #[oai(path = "/", method = "get")]
    pub async fn get_all_items(&self, db: Data<&Arc<Mutex<Db>>>) -> Result<FetchResponse<Vec<Item>>> {
        let db_ref = db
            .lock()
            .map_err(|_| GenericError::DbLock)?;

        let items = db_ref
            .find_all::<Item>(String::from(ITEM_TABLE_NAME))
            .unwrap_or_default();

        Ok(FetchResponse::Ok(Json(items)))
    }


    #[oai(path = "/:id", method = "get")]
    pub async fn get_item(&self, Path(id): Path<u32>, db: Data<&Arc<Mutex<Db>>>) -> Result<FetchResponse<Item>> {
        let db_ref = db
            .lock()
            .map_err(|_| GenericError::DbLock)?;

        let item = db_ref.find_by_id::<Item>(String::from(ITEM_TABLE_NAME), id)
            .ok_or(FetchResponse::not_found(id))?;

        Ok(FetchResponse::Ok(Json(item)))
    }


    #[protect("MUTATE")]
    #[oai(path = "/", method = "post")]
    pub async fn create_item(&self, db: Data<&Arc<Mutex<Db>>>, payload: Json<ItemCreateBody>) -> Result<CreateResponse<Item>> {
        let mut db_ref = db
        .lock()
        .map_err(|_| GenericError::DbLock)?;

        let id = db_ref
            .get_increment_last_id(ITEM_TABLE_NAME.to_string())
            .map_err(|_| GenericError::DbOperation)?
            .ok_or(GenericError::TableNotFound)?;

        let to_insert = Item::new(id, payload.0.name);
        let item = db_ref
            .insert_or_update(ITEM_TABLE_NAME.to_string(), id, to_insert)
            .map_err(|_| GenericError::DbOperation)?
            .ok_or(GenericError::TableNotFound)?;

        Ok(CreateResponse::Created(Json(item)))
    }

    #[protect("MUTATE")]
    #[oai(path = "/:id", method = "put")]
    pub async fn put_item(&self, Path(id): Path<u32>, payload: Json<ItemUpdateBody>, db: Data<&Arc<Mutex<Db>>>) -> Result<UpdateResponse<Item>> {
        let mut db_ref = db
            .lock()
            .map_err(|_| GenericError::DbLock)?;
    
        db_ref
            .find_by_id::<Item>(ITEM_TABLE_NAME.to_string(), id)
            .ok_or(UpdateResponse::not_found(id))?;

        let to_update = Item::new(id, payload.0.name);
        db_ref
            .insert_or_update(ITEM_TABLE_NAME.to_string(), id, to_update.clone())
            .map_err(|_| GenericError::DbOperation)?
            .ok_or(GenericError::TableNotFound)?;
    
        Ok(UpdateResponse::Ok(Json(to_update)))
    }

    #[protect("MUTATE")]
    #[oai(path = "/:id", method = "delete")]
    pub async fn delete_item(&self, Path(id): Path<u32>, db: Data<&Arc<Mutex<Db>>>) -> Result<DeleteResponse> {
        let mut db_ref = db
            .lock()
            .map_err(|_| GenericError::DbLock)?;

        db_ref
            .delete_by_id(ITEM_TABLE_NAME.to_string(), id)
            .map_err(|_| GenericError::DbOperation)?;

        Ok(DeleteResponse::success())
    }
}

#[cfg(test)]
mod tests {
    use poem::{http::StatusCode, Endpoint};

    use crate::test::{async_run_with_file_create_teardown, OpenApiTestClient};

    use super::*;

    fn insert_item(db: &mut Db, name: String) {
        let table_name = "item".to_string();
        db.add_table(table_name.clone(), false).unwrap();
        let id = db.get_increment_last_id(table_name.clone()).unwrap().unwrap();
        let to_insert = Item::new(id, name);
        db.insert_or_update(table_name.clone(), id, to_insert).unwrap();
    }

    fn init_api_client(file_name: String) -> OpenApiTestClient<impl Endpoint> {
        let test_client = OpenApiTestClient::init(ItemsApi, file_name.as_str());

        {
            let mut db = test_client.db.lock().unwrap();
            db.add_table("item".to_string(), false).unwrap();
            db.delete_all("item".to_string()).unwrap();
        }

        return test_client
    }

    #[tokio::test]
    async fn test_api_get_all_items() {
        async_run_with_file_create_teardown(|file_name| {
            async {
                let test_client = init_api_client(file_name);
                {
                    let mut db = test_client.db.lock().unwrap();
                    insert_item(&mut db, String::from("item 1"));
                    insert_item(&mut db, String::from("item 2"));
                    insert_item(&mut db, String::from("item 3"));
                }

                let response = test_client.client.get("/items")
                    .send()
                    .await;

                    let expected_data = serde_json::json!([
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
                    ]);
                
                response.assert_status_is_ok();
                response.assert_json(expected_data).await;
            }
        }).await;
    }

    #[tokio::test]
    async fn test_api_get_item_by_id() {
        async_run_with_file_create_teardown(|file_name| {
            async {
                let test_client = init_api_client(file_name);
                {
                    let mut db = test_client.db.lock().unwrap();
                    insert_item(&mut db, String::from("item 1"));
                    insert_item(&mut db, String::from("item 2"));
                    insert_item(&mut db, String::from("item 3"));
                }
        
                let response = test_client.client.get("/items/2").send().await;
        
                let expected_data = serde_json::json!({
                    "id": 2,
                    "name": "item 2"
                });
        
                response.assert_status_is_ok();
                response.assert_json(expected_data).await;
            }
        }).await;
    }

    #[tokio::test]
    async fn test_api_get_item_by_id_not_found() {
        async_run_with_file_create_teardown(|file_name| {
            async {      
                let test_client = init_api_client(file_name);
                let response = test_client.client.get("/items/99").send().await;
        
                response.assert_status(StatusCode::NOT_FOUND);
            }
        }).await;
    }

    #[tokio::test]
    async fn test_api_create_item() {
        async_run_with_file_create_teardown(|file_name| {
            async {      
                let test_client = init_api_client(file_name);
        
                let response = test_client.client.post("/items")
                    .body_json(&ItemCreateBody{ name: "item 1".to_string() })
                    .header("Authorization", format!("Bearer {}", test_client.token))
                    .send()
                    .await;
        
                let expected_data = serde_json::json!({
                    "id": 1,
                    "name": "item 1"
                });
        
                response.assert_status(StatusCode::CREATED);
                response.assert_json(expected_data).await;
            }
        }).await;
    }

    #[tokio::test]
    async fn test_api_put_item() {
        async_run_with_file_create_teardown(|file_name| {
            async {
                let test_client = init_api_client(file_name);

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
            }
        }).await;
    }

    #[tokio::test]
    async fn test_api_delete_item() {
        async_run_with_file_create_teardown(|file_name| {
            async {
                let test_client = init_api_client(file_name);

                let response = test_client.client.delete("/items/1")
                    .header("Authorization", format!("Bearer {}", test_client.token))
                    .send()
                    .await;

                response.assert_status_is_ok();
            }
        }).await;
    }
}