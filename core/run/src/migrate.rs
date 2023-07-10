use std::cmp::Ordering;
use std::sync::Arc;

use common_config_parser::types::Config;
use core_storage::{
    adapter::rocks::{ReadOnlyRocksAdapter, RocksAdapter},
    ImplStorage,
};
use protocol::{traits::VersionedStorage, types::RichBlock};

use crate::{migrations, MainError};

const INIT_DB_VERSION: &str = "20230706050403";

/// A helper struct for migrations.
pub struct Migrate {
    migrations: migrations::Migrations,
}

impl Migrate {
    /// Constructs a new struct.
    pub fn new() -> Self {
        let mut migrations = migrations::Migrations::new();
        migrations.push(Box::new(migrations::DefaultMigration::new(INIT_DB_VERSION)));
        Self { migrations }
    }

    /// Check if database's version is matched with the executable binary
    /// version.
    pub fn check(&self, adapter: &ReadOnlyRocksAdapter) -> Result<Ordering, MainError> {
        self.migrations
            .check(adapter)
            .map_err(|err| MainError::Other(err.to_string()))
    }

    /// Whether there is any migration requires confirmation from users.
    pub fn require_confirm(&self, adapter: &ReadOnlyRocksAdapter) -> Result<bool, MainError> {
        self.migrations
            .require_confirm(adapter)
            .map_err(|err| MainError::Other(err.to_string()))
    }

    /// Displays a note to warning users about the risks before they confirm the
    /// migration.
    pub fn notes(&self, adapter: &ReadOnlyRocksAdapter) -> Result<Vec<(&str, &str)>, MainError> {
        adapter
            .version()
            .map_err(|err| MainError::Other(err.to_string()))
            .map(|ver_opt| self.migrations.notes(ver_opt.as_ref()))
    }

    /// Initializes version for a database.
    pub fn initialize_version(
        &self,
        adapter: &Arc<ImplStorage<RocksAdapter>>,
    ) -> Result<(), MainError> {
        self.migrations
            .initialize_version(adapter)
            .map_err(|err| MainError::Other(err.to_string()))
    }

    /// Migrates the input database.
    pub async fn migrate(
        &self,
        adapter: &Arc<ImplStorage<RocksAdapter>>,
        config: &Config,
        genesis: &RichBlock,
    ) -> Result<(), MainError> {
        self.migrations
            .migrate(adapter, config, genesis)
            .await
            .map_err(|err| MainError::Other(err.to_string()))
    }
}
