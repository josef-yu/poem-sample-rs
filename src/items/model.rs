use serde::{Serialize, Deserialize};
use poem::{http::StatusCode, Error, FromRequest, Result};
use serde_json::Value;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: u32,
    pub name: String
}

impl Item {
    pub fn new(id: u32, name: String) -> Self {
        Self { id, name }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ItemCreateBody {
    pub name: String
}

impl<'a> FromRequest<'a> for ItemCreateBody {
    async fn from_request(
            _: &'a poem::Request,
            body: &mut poem::RequestBody,
        ) -> Result<Self> {
        let body = body
            .take()
            .unwrap()
            .into_json::<ItemCreateBody>()
            .await
            .map_err(|_| Error::from_string("Malformed body", StatusCode::BAD_REQUEST))?;

        Ok(body)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ItemUpdateBody {
    pub name: String
}

impl<'a> FromRequest<'a> for ItemUpdateBody {
    async fn from_request(
            _: &'a poem::Request,
            body: &mut poem::RequestBody,
        ) -> Result<Self> {
        let body = body
            .take()
            .unwrap()
            .into_json::<ItemUpdateBody>()
            .await
            .map_err(|_| Error::from_string("Malformed body", StatusCode::BAD_REQUEST))?;

        Ok(body)
    }
}

impl From<Value> for Item {
    fn from(value: Value) -> Self {
        serde_json::from_value::<Item>(value)
            .unwrap()
    }
}

impl From<Item> for Value {
    fn from(value: Item) -> Value {
        serde_json::to_value(value)
            .unwrap()
    }
}