use std::sync::{Arc, Mutex};

use poem::{handler, http::StatusCode, post, web::Data, Error, Result, Route};
use serde_json::Value;

use crate::{auth::model::{UserFormBody, LoginResponse, User}, db::Db, response::GenericResponse};

use super::jwt;

pub const USER_TABLE_NAME: &str = "user";

#[handler]
pub fn login(payload: UserFormBody, db: Data<&Arc<Mutex<Db>>>, manager: Data<&jwt::Manager>) -> Result<GenericResponse<LoginResponse>> {
    let db_ref = db
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
        .expect("Getting db lock");
    let user = db_ref.find_by_value::<User>(USER_TABLE_NAME.to_string(), "username".to_string(), payload.username)
        .map(|x| x.first().cloned().unwrap())
        .ok_or(
            Error::from_status(StatusCode::UNAUTHORIZED)
        )?;
    
    if user.password != payload.password {
        return Err(Error::from_status(StatusCode::UNAUTHORIZED))
    }

    let token_data = manager.create_token_data(user.username, user.permissions);
    let token = manager.encode(token_data)
        .expect("Encoding jwt");

    Ok(GenericResponse{
        status_code_u16: StatusCode::OK.as_u16(),
        message: None,
        data: Some(LoginResponse{ token })
    })
}

#[handler]
pub fn register(payload: UserFormBody, db: Data<&Arc<Mutex<Db>>>) -> Result<GenericResponse<Value>> {
    let mut db_ref = db
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
        .expect("Getting db lock");
    let users = db_ref
        .find_by_value::<User>(USER_TABLE_NAME.to_string(), "username".to_string(), payload.username.clone())
        .ok_or(
            Error::from_status(StatusCode::UNAUTHORIZED)
        )?;
    
    if !users.is_empty() {
        return Ok(GenericResponse{
            status_code_u16: StatusCode::BAD_REQUEST.as_u16(),
            message: Some("User already exists!".to_string()),
            data: None
        })
    }
    let id = db_ref.get_increment_last_id(USER_TABLE_NAME.to_string()).unwrap().unwrap();
    // Skipping hashing of password
    let to_insert = User::new(id, payload.username, payload.password, vec!["MUTATE".to_string()]);
    db_ref
        .insert_or_update(USER_TABLE_NAME.to_string(), id, to_insert)
        .unwrap()
        .unwrap();

    Ok(GenericResponse::<Value>{
        message: Some("User registered successfully.".to_string()),
        status_code_u16: StatusCode::CREATED.as_u16(),
        data: None
    })
}

pub fn auth_routes() -> Route {
    Route::new()
        .at("/login", post(login))
        .at("/register", post(register))
}


#[cfg(test)]
mod tests {
    use poem::Endpoint;

    use crate::test::{async_run_with_file_create_teardown, ApiTestClient, TEST_PASSWORD, TEST_USERNAME};

    use super::*;

    fn init_client() -> ApiTestClient<impl Endpoint> {
        let routes = Route::new().nest(
            "/", auth_routes()
        );
        let test_client = ApiTestClient::init(routes);
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
    async fn test_login() {
        async_run_with_file_create_teardown(|| async {
            let test_client = init_client();
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
        }).await;
    }

    #[tokio::test]
    async fn test_register() {
        async_run_with_file_create_teardown(|| async {
            let test_client = init_client();

            let response = test_client.client.post("/register")
                .body_json(&UserFormBody{ 
                    username: TEST_USERNAME.to_string(),
                    password: TEST_PASSWORD.to_string()
                })
                .send()
                .await;

            response.assert_status(StatusCode::CREATED);
        }).await;
    }
}