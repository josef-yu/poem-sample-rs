use serde::{Serialize, Deserialize};
use poem::{http::StatusCode, Error, FromRequest, Result};


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
            .or_else(|_| Err(Error::from_string("Malformed body", StatusCode::BAD_REQUEST)))?;

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
            .or_else(|_| Err(Error::from_string("Malformed body", StatusCode::BAD_REQUEST)))?;

        Ok(body)
    }
}