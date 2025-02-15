use std::collections::BTreeMap;

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
    perm::Perms,
    router::{InternalRouter, Router},
    PermsInstance,
};

#[derive(Serialize, Deserialize)]
pub struct ListReq {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ListRes {
    #[serde(rename = "list")]
    List { values: BTreeMap<String, u32> },
    #[serde(rename = "error")]
    Error { reason: String },
}

impl ListRes {
    pub fn success(values: BTreeMap<String, u32>) -> Self {
        Self::List { values }
    }

    pub fn failure(e: mongodb::error::Error) -> Self {
        Self::Error {
            reason: e
                .get_custom::<String>()
                .cloned()
                .unwrap_or(e.kind.to_string()),
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            ListRes::List { .. } => StatusCode::OK,
            ListRes::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl InternalRouter {
    pub async fn list(instance: &PermsInstance, payload: ListReq) -> ListRes {
        Perms::list(instance, &payload.id)
            .await
            .map(ListRes::success)
            .unwrap_or_else(ListRes::failure)
    }
}

impl Router {
    pub async fn list(
        State(instance): State<PermsInstance>,
        Json(payload): Json<ListReq>,
    ) -> (StatusCode, Json<ListRes>) {
        let res = InternalRouter::list(&instance, payload).await;
        (res.status(), Json(res))
    }
}
