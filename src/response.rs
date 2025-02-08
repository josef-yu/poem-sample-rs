use poem::{http::StatusCode, Body, IntoResponse, Response};
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

        if map.len() > 0 {
            return response.body(
                Body::from_json(map)
                    .unwrap()   
            )
        }

        response.finish()
    }
}