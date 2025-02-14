use axum::routing::post;

use crate::PermsInstance;

pub struct InternalRouter;
pub struct Router;

impl Router {
    pub fn get(instance: PermsInstance) -> axum::Router {
        axum::Router::new()
            .route("/show", post(Router::show))
            .route("/set", post(Router::set))
            .route("/list", post(Router::list))
            .route("/wipe", post(Router::wipe))
            .with_state(instance)
    }
}
