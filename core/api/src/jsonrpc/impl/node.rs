use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use core_consensus::SYNC_STATUS;
use protocol::lazy::CHAIN_ID;
use protocol::types::{Hash, Hasher, Hex, H160, H256, U256};

use crate::jsonrpc::{web3_types::Web3SyncStatus, AxonNodeRpcServer, RpcResult};

pub struct NodeRpcImpl {
    version: String,
    pprof:   Arc<AtomicBool>,
    _path:   PathBuf,
}

impl NodeRpcImpl {
    pub fn new(ver: String, path: PathBuf) -> Self {
        NodeRpcImpl {
            version: ver,
            pprof:   Arc::new(AtomicBool::new(false)),
            _path:   path.join("api"),
        }
    }
}

impl AxonNodeRpcServer for NodeRpcImpl {
    fn chain_id(&self) -> RpcResult<U256> {
        Ok((**CHAIN_ID.load()).into())
    }

    fn net_version(&self) -> RpcResult<String> {
        Ok((**CHAIN_ID.load()).to_string())
    }

    fn client_version(&self) -> RpcResult<String> {
        Ok(self.version.clone())
    }

    fn listening(&self) -> RpcResult<bool> {
        Ok(true)
    }

    fn syncing(&self) -> RpcResult<Web3SyncStatus> {
        Ok(SYNC_STATUS.read().clone().into())
    }

    fn mining(&self) -> RpcResult<bool> {
        Ok(false)
    }

    fn coinbase(&self) -> RpcResult<H160> {
        Ok(H160::default())
    }

    fn hashrate(&self) -> RpcResult<U256> {
        Ok(U256::one())
    }

    fn submit_work(&self, _nc: U256, _hash: H256, _summary: Hex) -> RpcResult<bool> {
        Ok(true)
    }

    fn submit_hashrate(&self, _hash_rate: Hex, _client_id: Hex) -> RpcResult<bool> {
        Ok(true)
    }

    fn pprof(&self, _enable: bool) -> RpcResult<bool> {
        #[cfg(feature = "pprof")]
        {
            use std::{
                fs::{create_dir_all, OpenOptions},
                time::Duration,
            };

            let old = self.pprof.load(Ordering::Acquire);
            self.pprof.store(_enable, Ordering::Release);
            if !old && _enable {
                let flag = Arc::clone(&self.pprof);
                let path = self._path.clone();

                std::thread::spawn(move || {
                    use pprof::protos::Message;
                    use std::io::Write;

                    let guard = pprof::ProfilerGuard::new(100).unwrap();
                    while flag.load(Ordering::Acquire) {
                        std::thread::sleep(Duration::from_secs(60));
                        if let Ok(report) = guard.report().build() {
                            create_dir_all(&path).unwrap();
                            let tmp_dir = path.join("tmp");
                            create_dir_all(&tmp_dir).unwrap();
                            let tmp_file = tmp_dir.join("profile.pb");
                            let mut file = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .append(false)
                                .open(&tmp_file)
                                .unwrap();
                            let profile = report.pprof().unwrap();

                            let mut content = Vec::new();
                            profile.encode(&mut content).unwrap();
                            file.write_all(&content).unwrap();
                            file.sync_all().unwrap();
                            move_file(tmp_file, path.join("profile.pb")).unwrap();
                        };
                    }
                });
            }
        }

        Ok(self.pprof.load(Ordering::Acquire))
    }

    fn sha3(&self, data: Hex) -> RpcResult<Hash> {
        Ok(Hasher::digest(data.as_ref()))
    }
}

/// This function use `copy` then `remove_file` as a fallback when `rename`
/// failed, this maybe happen when src and dst on different file systems.
#[cfg(feature = "pprof")]
fn move_file<P: AsRef<std::path::Path>>(src: P, dst: P) -> Result<(), std::io::Error> {
    use std::fs::{copy, remove_file, rename};

    if rename(&src, &dst).is_err() {
        copy(&src, &dst)?;
        remove_file(&src)?;
    }
    Ok(())
}
