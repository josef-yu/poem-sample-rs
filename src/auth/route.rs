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
        .or_else(|_| Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)))
        .expect("Getting db lock");
    let user = db_ref.find_by_value::<User>(USER_TABLE_NAME.to_string(), "username".to_string(), payload.username)
        .map(|x| x.first().cloned().unwrap())
        .ok_or(
            Error::from_status(StatusCode::UNAUTHORIZED)
        ).expect("Finding user by value");
    
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
    let mut db_ref = db.lock().or_else(|_| Err(Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)))?;
    let users = db_ref
        .find_by_value::<User>(USER_TABLE_NAME.to_string(), "username".to_string(), payload.username.clone())
        .ok_or(
            Error::from_status(StatusCode::UNAUTHORIZED)
        )?;
    
    if users.len() > 0 {
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
    return Route::new()
        .at("/login", post(login))
        .at("/register", post(register))
}


#[cfg(test)]
mod tests {
    use poem::{middleware::AddData, test::TestClient, EndpointExt, Middleware};

    use crate::{auth, test::{async_run_with_file_create_teardown, TEST_FILE_NAME}};

    use super::*;

    #[tokio::test]
    async fn test_login() {
        async_run_with_file_create_teardown(|| async {
            let routes = auth_routes();
            let mut db = Db::init(String::from(TEST_FILE_NAME)).unwrap();
            db.add_table(USER_TABLE_NAME.to_string(), true).unwrap();

            let username = "username".to_string();
            let password = "password".to_string();

            let id = db.get_increment_last_id(USER_TABLE_NAME.to_string()).unwrap().unwrap();
            let to_insert = User::new(
                id, 
                username.clone(), 
                password.clone(), 
                vec!["MUTATE".to_string()]
            );
            db
                .insert_or_update(USER_TABLE_NAME.to_string(), id, to_insert)
                .unwrap()
                .unwrap();

            let arc_db = Arc::new(Mutex::new(db));
            let jwt_manager = auth::jwt::Manager::init("secret".to_string(), 24);
            let client = TestClient::new(
                Route::new().nest("/", routes)
                    .with(AddData::new(arc_db).combine(AddData::new(jwt_manager)))
            );

            let response = client.post("/login")
                .body_json(&UserFormBody{
                    username,
                    password
                })
                .send()
                .await;
            
            response.assert_status_is_ok();
        }).await;
    }

    #[tokio::test]
    async fn test_register() {
        async_run_with_file_create_teardown(|| async {
            let routes = auth_routes();
            let mut db = Db::init(String::from(TEST_FILE_NAME)).unwrap();
            db.add_table(USER_TABLE_NAME.to_string(), false).unwrap();
            db.delete_all(USER_TABLE_NAME.to_string()).unwrap();
    
            let arc_db = Arc::new(Mutex::new(db));
    
            let client = TestClient::new(
                Route::new().nest("/", routes)
                    .with(AddData::new(arc_db))
            );

            let response = client.post("/register")
                .body_json(&UserFormBody{ 
                    username: "username".to_string(),
                    password: "password".to_string()
                })
                .send()
                .await;

            response.assert_status(StatusCode::CREATED);
        }).await;
    }
}