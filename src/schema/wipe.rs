use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
    perm::Perms,
    router::{InternalRouter, Router},
    PermsInstance,
};

#[derive(Serialize, Deserialize)]
pub struct WipeReq {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WipeRes {
    #[serde(rename = "wiped")]
    Wiped,
    #[serde(rename = "error")]
    Error { reason: String },
}

impl WipeRes {
    pub fn success(_: ()) -> Self {
        Self::Wiped
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
            WipeRes::Wiped => StatusCode::OK,
            WipeRes::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl InternalRouter {
    pub async fn wipe(instance: &PermsInstance, payload: WipeReq) -> WipeRes {
        Perms::wipe(instance, &payload.id)
            .await
            .map(WipeRes::success)
            .unwrap_or_else(WipeRes::failure)
    }
}

impl Router {
    pub async fn wipe(
        State(instance): State<PermsInstance>,
        Json(payload): Json<WipeReq>,
    ) -> (StatusCode, Json<WipeRes>) {
        let res = InternalRouter::wipe(&instance, payload).await;
        (res.status(), Json(res))
    }
}
