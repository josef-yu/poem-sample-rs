use poem::{http::StatusCode, Body, IntoResponse, Response};
use poem_openapi::{payload::Json, types::ToJSON, ApiResponse, Object};
use serde::Serialize;
use serde_json::{Map, Value};


#[derive(Serialize)]
pub struct GenericResponse<T> {
    pub status_code_u16: u16,
    pub message: Option<String>,
    pub data: Option<T>
}

impl<T> IntoResponse for GenericResponse<T> 
    where T: Serialize + Send + Into<Value>
{
    fn into_response(self) -> Response {
        let status_code = StatusCode::from_u16(self.status_code_u16)
            .unwrap();

        let response = Response::builder()
            .status(status_code)
            .content_type("application/json");

        let mut map = Map::new();

        if let Some(data) = self.data {
            map.insert("data".to_string(), data.into());
        }

        if let Some(message) = self.message {
            map.insert("message".to_string(), Value::String(message));
        }

        if !map.is_empty() {
            return response.body(
                Body::from_json(map)
                    .unwrap()   
            )
        }

        response.finish()
    }
}

#[derive(Debug, Object)]
pub struct Detail {
    pub message: String
}

#[derive(ApiResponse)]
pub enum FetchResponse<T: ToJSON> {
    #[oai(status = 200)]
    Ok(Json<T>),

    #[oai(status = 404)]
    NotFound(Json<Detail>)
}

#[derive(ApiResponse)]
pub enum DeleteResponse {
    #[oai(status = 200)]
    Ok(Json<Detail>),

    #[oai(status = 404)]
    NotFound(Json<Detail>)
}

#[derive(ApiResponse)]
pub enum CreateResponse<T: ToJSON> {
    #[oai(status = 201)]
    Created(Json<T>),

    #[oai(status = 200)]
    Ok(Json<T>),

    #[oai(status = 400)]
    BadRequest(Json<T>)
}

#[derive(ApiResponse)]
pub enum UpdateResponse<T: ToJSON> {
    #[oai(status = 200)]
    Ok(Json<T>),

    #[oai(status = 400)]
    NotFound(Json<Detail>)
}

#[derive(ApiResponse)]
pub enum GenericError {
    #[oai(status = 500)]
    DbLock,

    #[oai(status = 500)]
    Internal,

    #[oai(status = 500)]
    DbOperation,

    #[oai(status = 500)]
    TableNotFound,

    #[oai(status = 401)]
    Unauthorized(Json<Detail>),

    #[oai(status = 500)]
    JwtEncoding,

    #[oai(status = 400)]
    BadRequest(Json<Detail>)
}

impl GenericError {
    pub fn not_authorized() -> Self {
        let detail = Detail {
            message: "You are not authorized to do this request".to_string()
        };

        Self::Unauthorized(Json(detail))
    }
}