use std::collections::BTreeMap;

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
    perm::Perms,
    router::{InternalRouter, Router},
    PermsInstance,
};

#[derive(Serialize, Deserialize)]
pub struct ShowReq {
    pub id: String,
    pub entries: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ShowRes {
    #[serde(rename = "show")]
    Show { values: BTreeMap<String, u32> },
    #[serde(rename = "error")]
    Error { reason: String },
}

impl ShowRes {
    pub fn success(values: BTreeMap<String, u32>) -> Self {
        Self::Show { values }
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
            ShowRes::Show { .. } => StatusCode::OK,
            ShowRes::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl InternalRouter {
    pub async fn show(instance: &PermsInstance, payload: ShowReq) -> ShowRes {
        Perms::get(instance, &payload.id, payload.entries)
            .await
            .map(ShowRes::success)
            .unwrap_or_else(ShowRes::failure)
    }
}

impl Router {
    pub async fn show(
        State(instance): State<PermsInstance>,
        Json(payload): Json<ShowReq>,
    ) -> (StatusCode, Json<ShowRes>) {
        let res = InternalRouter::show(&instance, payload).await;
        (res.status(), Json(res))
    }
}
