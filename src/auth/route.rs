use std::sync::{Arc, Mutex};

use poem::{web::Data, Result};
use poem_openapi::{payload::Json, OpenApi};

use crate::{auth::model::{LoginResponse, User, UserFormBody}, db::Db, response::{CreateResponse, Detail, GenericError}};

use super::{jwt, model::RegisterReponse};

pub const USER_TABLE_NAME: &str = "user";

pub struct AuthApi;

#[poem_grants::open_api]
#[OpenApi]
impl AuthApi {

    #[oai(path = "/login", method = "post")]
    pub async fn login(&self, payload: UserFormBody, db: Data<&Arc<Mutex<Db>>>, manager: Data<&jwt::Manager>) -> Result<CreateResponse<LoginResponse>> {
        let db_ref = db
            .lock()
            .map_err(|_| GenericError::DbLock)?;

        let user = db_ref.find_by_value::<User>(USER_TABLE_NAME.to_string(), "username".to_string(), payload.username)
            .map(|x| x.first().cloned().unwrap())
            .ok_or(GenericError::not_authorized())?;
        
        if user.password != payload.password {
            return Err(GenericError::not_authorized().into())
        }

        let token_data = manager.create_token_data(user.username, user.permissions);
        let token = manager.encode(token_data)
            .map_err(|_| GenericError::JwtEncoding)?;

        Ok(LoginResponse::new(token))
    }

    #[oai(path = "/register", method = "post")]
    pub async fn register(&self, payload: Json<UserFormBody>, db: Data<&Arc<Mutex<Db>>>) -> Result<CreateResponse<Detail>> {
        let mut db_ref = db
            .lock()
            .map_err(|_| GenericError::DbLock)?;

        let users = db_ref
            .find_by_value::<User>(USER_TABLE_NAME.to_string(), "username".to_string(), payload.0.username.clone())
            .ok_or(GenericError::DbOperation)?;
        
        if !users.is_empty() {
            let detail = Detail {
                message: "User already exists!".to_string()
            };

            return Err(GenericError::BadRequest(Json(detail)).into())
        }
        
        let id = db_ref
            .get_increment_last_id(USER_TABLE_NAME.to_string())
            .map_err(|_| GenericError::DbOperation)?
            .ok_or(GenericError::TableNotFound)?;
        
        // Skipping hashing of password
        let to_insert = User::new(id, payload.0.username, payload.0.password, vec!["MUTATE".to_string()]);
        
        let inserted_user = db_ref
            .insert_or_update(USER_TABLE_NAME.to_string(), id, to_insert)
            .map_err(|_| GenericError::DbOperation)?;

        if inserted_user.is_none() {
            return Err(GenericError::DbOperation.into())
        }

        Ok(CreateResponse::success())
    }
}


#[cfg(test)]
mod tests {
    use poem::{http::StatusCode, Endpoint};

    use crate::test::{async_run_with_file_create_teardown, OpenApiTestClient, TEST_PASSWORD, TEST_USERNAME};

    use super::*;

    fn init_api_client(file_name: String) -> OpenApiTestClient<impl Endpoint> {
        let test_client = OpenApiTestClient::init(AuthApi, file_name.as_str());
        {
            let mut db = test_client.db.lock().unwrap();
            db.add_table(USER_TABLE_NAME.to_string(), false).unwrap();
            db.delete_all(USER_TABLE_NAME.to_string()).unwrap();
        }

        return test_client
    }

    fn insert_user(db: &mut Db, username: &str, password: &str) {
        let id = db.get_increment_last_id(USER_TABLE_NAME.to_string()).unwrap().unwrap();
        let to_insert = User::new(
            id, 
            username.to_string(), 
            password.to_string(), 
            vec!["MUTATE".to_string()]
        );
        db
            .insert_or_update(USER_TABLE_NAME.to_string(), id, to_insert)
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn test_api_login() {
        async_run_with_file_create_teardown(|file_name| {
            async {
                let test_client = init_api_client(file_name);
                {
                    let mut db = test_client.db.lock().unwrap();
                    insert_user(&mut db, TEST_USERNAME, TEST_PASSWORD);
                }

                let response = test_client.client.post("/login")
                    .body_json(&UserFormBody{
                        username: TEST_USERNAME.to_string(),
                        password: TEST_PASSWORD.to_string()
                    })
                    .send()
                    .await;
                
                response.assert_status_is_ok();
            }
        }).await;
    }

    #[tokio::test]
    async fn test_api_register() {
        async_run_with_file_create_teardown(|file_name| {
            async {
                let test_client = init_api_client(file_name);

                let response = test_client.client.post("/register")
                    .body_json(&UserFormBody{ 
                        username: TEST_USERNAME.to_string(),
                        password: TEST_PASSWORD.to_string()
                    })
                    .send()
                    .await;

                response.assert_status(StatusCode::CREATED);
            }
        }).await;
    }
}