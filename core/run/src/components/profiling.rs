//! Control the profiling related features.

use std::sync::Arc;

use common_memory_tracker::{GetColumnFamilys, GetProperty, GetPropertyCF};
use jemalloc_ctl::{Access, AsName};
use jemallocator::Jemalloc;
use protocol::tokio;

#[global_allocator]
pub static JEMALLOC: Jemalloc = Jemalloc;

pub(crate) fn start() {
    set_profile(true);
}

pub(crate) fn stop() {
    set_profile(false);
    dump_profile();
}

pub(crate) fn track_current_process() {
    tokio::spawn(common_memory_tracker::track_current_process());
}

pub(crate) fn track_db_process<DB>(typ: &'static str, db_ref: &Arc<DB>)
where
    DB: GetColumnFamilys + GetProperty + GetPropertyCF + Send + Sync + 'static,
{
    let db = Arc::clone(db_ref);
    tokio::spawn(common_memory_tracker::track_db_process::<DB>(typ, db));
}

fn set_profile(is_active: bool) {
    let _ = b"prof.active\0"
        .name()
        .write(is_active)
        .map_err(|e| panic!("Set jemalloc profile error {:?}", e));
}

fn dump_profile() {
    let name = b"profile.out\0".as_ref();
    b"prof.dump\0"
        .name()
        .write(name)
        .expect("Should succeed to dump profile")
}
