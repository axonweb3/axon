use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::Arc;

use console::Term;
use core_storage::{
    adapter::rocks::{ReadOnlyRocksAdapter, RocksAdapter, RocksAdapterError},
    ImplStorage,
};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget};

use common_config_parser::types::Config;
use protocol::{async_trait, traits::VersionedStorage as _, types::RichBlock, ProtocolError};

const NO_VERSION: &str = "00000000000000";

/// A single migration.
#[async_trait]
pub trait Migration {
    /// Applies the current migration.
    async fn migrate(
        &self,
        _storage: &Arc<ImplStorage<RocksAdapter>>,
        _config: &Config,
        _genesis: &RichBlock,
        _pb: Arc<dyn Fn(u64) -> ProgressBar + Send + Sync>,
    ) -> Result<(), ProtocolError>;

    /// Returns migration version, use `%Y%m%d%H%M%S` format.
    fn version(&self) -> &str;

    /// Whether current migration will cost a lot of time to perform.
    fn is_expensive(&self) -> bool;

    /// If there is anything important and users should know it before do
    /// migration, the put the note here.
    ///
    /// For example, if the current migration could corrupt the database
    /// potentially, this method should return a note to let users know
    /// that.
    fn note(&self) -> Option<&str>;
}

/// A list of migrations that should be performed in order.
#[derive(Default)]
pub struct Migrations {
    migrations: BTreeMap<String, Box<dyn Migration>>,
}

impl Migrations {
    /// Creates an empty list of migrations.
    pub fn new() -> Self {
        Migrations {
            migrations: BTreeMap::new(),
        }
    }

    /// Adds a migration at the last of current migrations list.
    pub fn push(&mut self, migration: Box<dyn Migration>) {
        self.migrations
            .insert(migration.version().to_string(), migration);
    }

    /// Check if database's version is matched with the executable binary
    /// version.
    ///
    /// Returns
    /// - Less: The database version is less than the matched version of the
    ///   executable binary. Requires migration.
    /// - Equal: The database version is matched with the executable binary
    ///   version.
    /// - Greater: The database version is greater than the matched version of
    ///   the executable binary. Requires upgrade the executable binary.
    pub fn check(&self, storage: &ReadOnlyRocksAdapter) -> Result<Ordering, RocksAdapterError> {
        let status = if let Some(version) = storage.version()? {
            if let Some(m) = self.migrations.values().last() {
                version.as_str().cmp(m.version())
            } else {
                Ordering::Equal
            }
        } else {
            Ordering::Less
        };
        Ok(status)
    }

    /// Whether there is any migration requires confirmation from users.
    pub fn require_confirm(
        &self,
        storage: &ReadOnlyRocksAdapter,
    ) -> Result<bool, RocksAdapterError> {
        let require_confirm = if let Some(version) = storage.version()? {
            self.migrations
                .values()
                .skip_while(|m| m.version() <= version.as_str())
                .any(|m| m.is_expensive() || m.note().is_some())
        } else {
            self.migrations
                .values()
                .any(|m| m.is_expensive() || m.note().is_some())
        };
        Ok(require_confirm)
    }

    /// A list of important notes that users should know them before perform
    /// migrations.
    pub fn notes(&self, version_opt: Option<&String>) -> Vec<(&str, &str)> {
        if let Some(version) = version_opt {
            self.migrations
                .values()
                .skip_while(|m| m.version() <= version.as_str())
                .filter_map(|m| m.note().map(|note| (m.version(), note)))
                .collect()
        } else {
            self.migrations
                .values()
                .filter_map(|m| m.note().map(|note| (m.version(), note)))
                .collect()
        }
    }

    async fn run(
        &self,
        storage: &Arc<ImplStorage<RocksAdapter>>,
        current_version: &str,
        config: &Config,
        genesis: &RichBlock,
    ) -> Result<(), ProtocolError> {
        let mpb = Arc::new(MultiProgress::new());
        let migrations: Vec<_> = self
            .migrations
            .iter()
            .filter_map(|(v, m)| {
                if v.as_str() > current_version {
                    Some(m)
                } else {
                    None
                }
            })
            .collect();
        let migrations_count = migrations.len();
        for (idx, m) in migrations.iter().enumerate() {
            let mpbc = Arc::clone(&mpb);
            let pb = move |count: u64| -> ProgressBar {
                let pb = mpbc.add(ProgressBar::new(count));
                pb.set_draw_target(ProgressDrawTarget::term(Term::stdout(), None));
                pb.set_prefix(format!("[{}/{}]", idx + 1, migrations_count));
                pb
            };
            m.migrate(storage, config, genesis, Arc::new(pb)).await?;
            storage.set_version(m.version())?;
        }
        mpb.join_and_clear().map_err(|_| {
            let errmsg = "failed to let MultiProgress join then clear".to_string();
            RocksAdapterError::Other(errmsg)
        })?;
        Ok(())
    }

    /// Initializes version for a database.
    pub fn initialize_version(
        &self,
        storage: &Arc<ImplStorage<RocksAdapter>>,
    ) -> Result<(), RocksAdapterError> {
        if storage.version()?.is_none() {
            if let Some(m) = self.migrations.values().last() {
                storage.set_version(m.version())?;
            }
        }
        Ok(())
    }

    /// Migrates the input database.
    pub async fn migrate(
        &self,
        storage: &Arc<ImplStorage<RocksAdapter>>,
        config: &Config,
        genesis: &RichBlock,
    ) -> Result<(), ProtocolError> {
        match storage.version()? {
            Some(ref v) => {
                if let Some(m) = self.migrations.values().last() {
                    if m.version() < v.as_str() {
                        log::error!(
                            "Database downgrade detected. \
                            The database schema version is newer than \
                            the schema version which the executable file uses,\
                            please upgrade the executable file to a newer version"
                        );
                        let err = RocksAdapterError::other("database downgrade is not supported");
                        return Err(err.into());
                    }
                }

                self.run(storage, v.as_str(), config, genesis).await?;
                Ok(())
            }
            None => {
                self.run(storage, NO_VERSION, config, genesis).await?;
                Ok(())
            }
        }
    }
}

/// A migration which do nothing.
pub struct DefaultMigration {
    version: String,
}

impl DefaultMigration {
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_owned(),
        }
    }
}

#[async_trait]
impl Migration for DefaultMigration {
    async fn migrate(
        &self,
        _storage: &Arc<ImplStorage<RocksAdapter>>,
        _config: &Config,
        _genesis: &RichBlock,
        _pb: Arc<dyn Fn(u64) -> ProgressBar + Send + Sync>,
    ) -> Result<(), ProtocolError> {
        Ok(())
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn is_expensive(&self) -> bool {
        false
    }

    fn note(&self) -> Option<&str> {
        None
    }
}
