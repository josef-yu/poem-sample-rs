use poem_openapi::{payload::Json, Object};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::response::{DeleteResponse, Detail, FetchResponse, UpdateResponse};


#[derive(Serialize, Deserialize, Debug, Clone, Object)]
pub struct Item {
    pub id: u32,
    pub name: String
}

impl Item {
    pub fn new(id: u32, name: String) -> Self {
        Self { id, name }
    }
}

#[derive(Serialize, Deserialize, Object)]
pub struct ItemCreateBody {
    pub name: String
}

#[derive(Serialize, Deserialize, Object)]
pub struct ItemUpdateBody {
    pub name: String
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

pub trait ItemNotFound {
    fn not_found(id: u32) -> Self;
}

impl ItemNotFound for FetchResponse<Item> {
    fn not_found(id: u32) -> Self {
        let detail = Detail {
            message: format!("Item {:?} not found.", id)
        };

        Self::NotFound(Json(detail))
    }
}

impl ItemNotFound for UpdateResponse<Item> {
    fn not_found(id: u32) -> Self{
        let detail = Detail {
            message: format!("Item {:?} not found.", id)
        };

        Self::NotFound(Json(detail))
    }
}

impl ItemNotFound for DeleteResponse {
    fn not_found(id: u32) -> Self {
        let detail = Detail {
            message: format!("Item {:?} not found.", id)
        };

        Self::NotFound(Json(detail))
    }
}

pub trait ItemDelete {
    fn success() -> Self;
}

impl ItemDelete for DeleteResponse {
    fn success() -> Self {
        let detail = Detail {
            message: "Item deleted successfully.".to_string()
        };

        Self::Ok(Json(detail))
    }
}