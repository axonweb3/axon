pub(crate) mod extensions;
pub(crate) mod network;
pub(crate) mod storage;
pub(crate) mod system;

#[cfg(all(
    not(target_env = "msvc"),
    not(target_os = "macos"),
    feature = "jemalloc"
))]
pub(crate) mod profiling;

#[cfg(not(all(
    not(target_env = "msvc"),
    not(target_os = "macos"),
    feature = "jemalloc"
)))]
pub(crate) mod profiling {
    use std::sync::Arc;

    pub(crate) fn start() {
        log::warn!("profiling is not supported, so it doesn't start");
    }
    pub(crate) fn stop() {
        log::warn!("profiling is not supported, so it doesn't require stopping");
    }
    pub(crate) fn track_current_process() {
        log::warn!("profiling is not supported, so it doesn't track current process");
    }
    pub(crate) fn track_db_process<DB>(typ: &str, _db: &Arc<DB>) {
        log::warn!("profiling is not supported, so it doesn't track db process for [{typ}]");
    }
}
