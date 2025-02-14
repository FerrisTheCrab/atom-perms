use std::path::Path;

use mongodb::{bson::Document, Collection};

use crate::MasterConfig;

#[derive(Clone)]
pub struct PermsInstance {
    pub config: MasterConfig,
    pub perms: Collection<Document>,
}

impl PermsInstance {
    pub fn load(config: &Path) -> Self {
        let config = MasterConfig::read(config);
        let perms = config.mongodb.load();
        PermsInstance { config, perms }
    }
}
