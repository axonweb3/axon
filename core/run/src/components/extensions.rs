//! Optional extensions.

use common_apm::{server::run_prometheus_server, tracing::global_tracer_register};
use common_config_parser::types::{ConfigJaeger, ConfigPrometheus};
use protocol::{tokio, ProtocolResult};

pub(crate) trait ExtensionConfig {
    const NAME: &'static str;

    /// Try to start and return the result.
    fn try_to_start(&self) -> ProtocolResult<bool>;

    /// Try to start and ignore the result.
    fn start_if_possible(&self) {
        match self.try_to_start() {
            Ok(started) => {
                if started {
                    log::info!("{} is started", Self::NAME);
                } else {
                    log::info!("{} is disabled", Self::NAME);
                }
            }
            Err(err) => {
                log::error!("failed to start {} since {err}", Self::NAME);
            }
        }
    }
}

impl ExtensionConfig for Option<ConfigJaeger> {
    const NAME: &'static str = "Jaeger";

    fn try_to_start(&self) -> ProtocolResult<bool> {
        if let Some(ref config) = self {
            if let Some(ref addr) = config.tracing_address {
                let service_name = config
                    .service_name
                    .as_ref()
                    .map(ToOwned::to_owned)
                    .unwrap_or("axon".to_owned());
                let tracing_batch_size = config.tracing_batch_size.unwrap_or(50);
                global_tracer_register(&service_name, addr.to_owned(), tracing_batch_size);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
}

impl ExtensionConfig for Option<ConfigPrometheus> {
    const NAME: &'static str = "Prometheus";

    fn try_to_start(&self) -> ProtocolResult<bool> {
        if let Some(ref config) = self {
            if let Some(ref addr) = config.listening_address {
                tokio::spawn(run_prometheus_server(addr.to_owned()));
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
}
