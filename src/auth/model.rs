use poem_openapi::{payload::Json, Object};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::response::{CreateResponse, Detail};


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

#[derive(Deserialize, Serialize, Object)]
pub struct UserFormBody {
    pub username: String,
    pub password: String
}

#[derive(Serialize, Deserialize, Object)]
pub struct LoginResponse {
    pub token: String
}

impl LoginResponse {
    pub fn new(token: String) -> CreateResponse<LoginResponse> {
        let body = LoginResponse{ token };

        CreateResponse::Ok(Json(body))
    }
}

impl From<LoginResponse> for Value {
    fn from(value: LoginResponse) -> Self {
        serde_json::to_value(value).unwrap()
    }
}

pub trait RegisterReponse {
    fn success() -> Self;
}

impl RegisterReponse for CreateResponse<Detail> {
    fn success() -> Self {
        let detail = Detail {
            message: "User registered successfully".to_string()
        };

        Self::Created(Json(detail))
    }
}