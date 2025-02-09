use poem::{http::StatusCode, Error, FromRequest, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub password: String,
    pub permissions: Vec<String>
}

impl User {
    pub fn new(id: u32, username: String, password: String, permissions: Vec<String>) -> Self {
        Self {
            id,
            username,
            password,
            permissions
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct UserFormBody {
    pub username: String,
    pub password: String
}

impl<'a> FromRequest<'a> for UserFormBody {
    async fn from_request(
            _: &'a poem::Request,
            body: &mut poem::RequestBody,
        ) -> Result<Self> {
            let body = body
                .take()
                .unwrap()
                .into_json::<UserFormBody>()
                .await
                .map_err(|_| Error::from_string("Malformed body", StatusCode::BAD_REQUEST))?;

        Ok(body)
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String
}

impl From<LoginResponse> for Value {
    fn from(value: LoginResponse) -> Self {
        serde_json::to_value(value).unwrap()
    }
}