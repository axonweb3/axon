use tiny_keccak::{Hasher, Keccak};

#[cfg(feature = "hash")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "hash")))]
pub fn keccak_256(data: &[u8]) -> [u8; 32] {
    let mut ret = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(data);
    hasher.finalize(&mut ret);
    ret
}

#[derive(Default)]
pub(crate) struct InnerKeccak;

impl cita_trie::Hasher for InnerKeccak {
    const LENGTH: usize = 32;

    fn digest(&self, data: &[u8]) -> Vec<u8> {
        keccak_256(data).to_vec()
    }
}
