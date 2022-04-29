use std::{sync::Arc, time::Duration};

use jemalloc_ctl::{epoch, stats};
use log::error;

use core_storage::{GetColumnFamilys, GetProperty, GetPropertyCF};
use protocol::tokio;

macro_rules! je_mib {
    ($key:ty) => {
        if let Ok(value) = <$key>::mib() {
            value
        } else {
            error!("failed to lookup jemalloc mib for {}", stringify!($key));
            return;
        }
    };
}

macro_rules! mib_read {
    ($mib:ident) => {
        if let Ok(value) = $mib.read() {
            value as i64
        } else {
            error!("failed to read jemalloc stats for {}", stringify!($mib));
            return;
        }
    };
}

macro_rules! metrics {
    ($label:literal, $type:literal, $value:expr) => {{
        common_apm::metrics::mem_tracker::MEMORY_TRACE_VEC
            .with_label_values(&[$label, $type])
            .set($value);
    }};
}

macro_rules! metrics_db {
    ($label:expr, $key:expr, $cf:expr, $value:expr) => {{
        common_apm::metrics::mem_tracker::DB_MEMORY_TRACE_VEC
            .with_label_values(&[$label, $key, $cf])
            .set($value);
    }};
}

pub async fn track_db_process<DB>(typ: &str, db: Arc<DB>)
where
    DB: GetColumnFamilys + GetProperty + GetPropertyCF + Send,
{
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;

        let db_type = format!("mem.rocksdb.{}", typ);

        // References: [Memory usage in RocksDB](https://github.com/facebook/rocksdb/wiki/Memory-usage-in-RocksDB)
        get_db_values(&db_type, "estimate-table-readers-mem", &*db);
        get_db_values(&db_type, "size-all-mem-tables", &*db);
        get_db_values(&db_type, "cur-size-all-mem-tables", &*db);
        get_db_values(&db_type, "block-cache-capacity", &*db);
        get_db_values(&db_type, "block-cache-usage", &*db);
        get_db_values(&db_type, "block-cache-pinned-usage", &*db);
    }
}

fn get_db_values<DB>(typ: &str, key: &str, db: &DB)
where
    DB: GetColumnFamilys + GetProperty + GetPropertyCF,
{
    for (cf_name, cf) in db.get_cfs() {
        let value = match db.property_int_value_cf(cf, &format!("rocksdb.{}", key)) {
            Ok(Some(v)) => v as i64,
            Ok(None) => -1,
            Err(_) => -2,
        };

        metrics_db!(typ, key, cf_name, value)
    }
}

pub async fn track_current_process() {
    let je_epoch = je_mib!(epoch);
    // Bytes allocated by the application.
    let allocated = je_mib!(stats::allocated);
    // Bytes in physically resident data pages mapped by the allocator.
    let resident = je_mib!(stats::resident);
    // Bytes in active pages allocated by the application.
    let active = je_mib!(stats::active);
    // Bytes in active extents mapped by the allocator.
    let mapped = je_mib!(stats::mapped);
    // Bytes in virtual memory mappings that were retained
    // rather than being returned to the operating system
    let retained = je_mib!(stats::retained);
    // Bytes dedicated to jemalloc metadata.
    let metadata = je_mib!(stats::metadata);

    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;

        if je_epoch.advance().is_err() {
            error!("failed to refresh the jemalloc stats");
            return;
        }

        if let Ok(memory) = process::get_current_process_memory() {
            // Resident set size, amount of non-swapped physical memory.
            let rss = memory.resident as i64 / 1024i64 / 1024i64;
            // Virtual memory size, total amount of memory.
            let vms = memory.size as i64 / 1024i64 / 1024i64;

            metrics!("mem.process", "rss", rss);
            metrics!("mem.process", "vms", vms);

            let allocated = mib_read!(allocated);
            let resident = mib_read!(resident);
            let active = mib_read!(active);
            let mapped = mib_read!(mapped);
            let retained = mib_read!(retained);
            let metadata = mib_read!(metadata);

            metrics!("mem.jemalloc", "allocated", allocated);
            metrics!("mem.jemalloc", "resident", resident);
            metrics!("mem.jemalloc", "active", active);
            metrics!("mem.jemalloc", "mapped", mapped);
            metrics!("mem.jemalloc", "retained", retained);
            metrics!("mem.jemalloc", "metadata", metadata);
        } else {
            error!("failed to fetch the memory information about current process");
        }
    }
}

pub mod process {
    use std::{fs, io, str::FromStr};

    #[derive(Debug)]
    pub struct Memory {
        // Virtual memory size
        pub size:     u64,
        // Size of physical memory being used
        pub resident: u64,
        // Number of shared pages
        pub _shared:  u64,
        // The size of executable virtual memory owned by the program
        pub _text:    u64,
        // Size of the program data segment and the user state stack
        pub _data:    u64,
    }

    impl FromStr for Memory {
        type Err = io::Error;

        fn from_str(value: &str) -> Result<Memory, io::Error> {
            static PAGE_SIZE: once_cell::sync::OnceCell<u64> = once_cell::sync::OnceCell::new();
            let page_size =
                PAGE_SIZE.get_or_init(|| unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 });
            let mut parts = value.split_ascii_whitespace();
            let size = parts
                .next()
                .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
                .and_then(|value| {
                    u64::from_str(value)
                        .map(|value| value * *page_size)
                        .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
                })?;
            let resident = parts
                .next()
                .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
                .and_then(|value| {
                    u64::from_str(value)
                        .map(|value| value * *page_size)
                        .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
                })?;
            let _shared = parts
                .next()
                .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
                .and_then(|value| {
                    u64::from_str(value)
                        .map(|value| value * *page_size)
                        .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
                })?;
            let _text = parts
                .next()
                .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
                .and_then(|value| {
                    u64::from_str(value)
                        .map(|value| value * *page_size)
                        .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
                })?;
            // ignore the size of the library in the virtual memory space of the task being
            // imaged
            let _lrs = parts.next();
            let _data = parts
                .next()
                .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
                .and_then(|value| {
                    u64::from_str(value)
                        .map(|value| value * *page_size)
                        .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
                })?;
            Ok(Memory {
                size,
                resident,
                _shared,
                _text,
                _data,
            })
        }
    }

    pub fn get_current_process_memory() -> Result<Memory, io::Error> {
        static PID: once_cell::sync::OnceCell<libc::pid_t> = once_cell::sync::OnceCell::new();
        let pid = PID.get_or_init(|| unsafe { libc::getpid() });
        let content = fs::read_to_string(format!("/proc/{}/statm", pid))?;

        Memory::from_str(&content)
    }
}
