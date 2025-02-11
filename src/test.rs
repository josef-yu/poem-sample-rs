use std::future::Future;
use std::panic::{self, AssertUnwindSafe};
use std::fs::File;
use std::sync::{Arc, Mutex};

use futures::FutureExt;
use poem::middleware::{AddData, Middleware};
use poem::test::TestClient;
use poem::{Endpoint, EndpointExt, IntoEndpoint, Route};
use poem_openapi::OpenApiService;
use serde_json::Value;
use uuid::Uuid;

use crate::auth;
use crate::db::Db;
use crate::response::GenericResponse;


pub static TEST_FILE_NAME: &str = "test-data.json";
pub const TEST_USERNAME: &str = "username";
pub const TEST_PASSWORD: &str = "password";
pub const TEST_PERMISSION: &str = "MUTATE";


pub fn run_with_file_create_teardown<T>(test: T)
    where T: FnOnce(String) + panic::UnwindSafe
{
    let uuid = Uuid::new_v4().to_string();
    let file_name = format!("./{}-{}", uuid, TEST_FILE_NAME);

    let _ = File::create(file_name.clone());

    let result = panic::catch_unwind(|| {
        test(file_name.clone())
    });

    let _ = std::fs::remove_file(file_name.clone());

    assert!(result.is_ok())
}


pub async fn async_run_with_file_create_teardown<T, U>(test: T)
    where T: FnOnce(String) -> U + panic::UnwindSafe,
        U: Future<Output = ()>
{
    let uuid = Uuid::new_v4().to_string();
    let file_name = format!("./{}-{}", uuid, TEST_FILE_NAME);

    let _ = File::create(file_name.clone());

    let result = AssertUnwindSafe(test(file_name.clone()))
        .catch_unwind()
        .await;

    let _ = std::fs::remove_file(file_name.clone());

    assert!(result.is_ok())
}

pub struct OpenApiTestClient<E> {
    pub db: Arc<Mutex<Db>>,
    pub client: TestClient<E>,
    pub jwt_manager: auth::jwt::Manager,
    pub token: String
}

impl<E: Endpoint + EndpointExt + 'static > OpenApiTestClient<E> {
    pub fn init<T>(api: T, file_name: &str) -> OpenApiTestClient<impl Endpoint + EndpointExt> 
        where OpenApiService<T, ()>: IntoEndpoint<Endpoint = E>
    {
        let db = Db::init(file_name.to_string()).unwrap();
        let arc_db = Arc::new(Mutex::new(db));
        
        let jwt_manager = auth::jwt::Manager::init("secret".to_string(), 24);
        let jwt_middleware = auth::middleware::JwtMiddleware{ manager: jwt_manager.clone() };
        let jwt_data = jwt_manager.create_token_data(TEST_USERNAME.to_string(), vec![TEST_PERMISSION.to_string()]);
        let token = jwt_manager.encode(jwt_data).unwrap();

        let api_service = OpenApiService::new(api, "test api", "1");

        let client = TestClient::new(
        Route::new()
            .nest("", api_service)
            .with(
    jwt_middleware
                    .combine(AddData::new(arc_db.clone()))
                    .combine(AddData::new(jwt_manager.clone()))
            )
            .catch_all_error(|err| async move {
                GenericResponse::<Value>{ 
                    message: Some(err.to_string()),
                    status_code_u16: err.status().as_u16(),
                    data: None
                }
            })
        );

        OpenApiTestClient {
            db: arc_db,
            jwt_manager,
            client,
            token
        }
    }
}