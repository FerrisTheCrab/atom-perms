use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
    perm::Perms,
    router::{InternalRouter, Router},
    PermsInstance,
};

#[derive(Serialize, Deserialize)]
pub struct SetEntry {
    pub key: String,
    pub value: u32,
}

impl SetEntry {
    pub fn into_tuple(self) -> (String, u32) {
        (self.key, self.value)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SetReq {
    pub id: String,
    pub entries: Vec<SetEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SetRes {
    #[serde(rename = "set")]
    Set,
    #[serde(rename = "error")]
    Error { reason: String },
}

impl SetRes {
    pub fn success(_: ()) -> Self {
        Self::Set
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
            SetRes::Set => StatusCode::OK,
            SetRes::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl InternalRouter {
    pub async fn set(instance: &PermsInstance, payload: SetReq) -> SetRes {
        Perms::set(
            instance,
            &payload.id,
            payload
                .entries
                .into_iter()
                .map(SetEntry::into_tuple)
                .collect(),
        )
        .await
        .map(SetRes::success)
        .unwrap_or_else(SetRes::failure)
    }
}

impl Router {
    pub async fn set(
        State(instance): State<PermsInstance>,
        Json(payload): Json<SetReq>,
    ) -> (StatusCode, Json<SetRes>) {
        let res = InternalRouter::set(&instance, payload).await;
        (res.status(), Json(res))
    }
}
