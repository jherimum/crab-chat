use crate::recipes::Recipe;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListMode {
    ALL,
    One(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    pub mode: ListMode,
    pub data: Vec<Recipe>,
    pub receiver: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRequest {
    pub mode: ListMode,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RecipeMessage {
    Request(ListRequest),
    Response(ListResponse),
}

impl RecipeMessage {
    pub fn serialize_to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error + Send + Sync + 'static>> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn deserialize_from_bytes(
        data: &[u8],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        Ok(serde_json::from_slice(data)?)
    }
}
