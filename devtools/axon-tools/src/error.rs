use std::fmt::{self, Display};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    InvalidProofBlockHash,
    NotEnoughSignatures,
    VerifyMptProof,
    HexPrefix,

    #[cfg(feature = "hex")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "hex")))]
    Hex(faster_hex::Error),

    #[cfg(feature = "proof")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "proof")))]
    Bls(blst::BLST_ERROR),

    #[cfg(feature = "proof")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "proof")))]
    Trie(cita_trie::TrieError),
}

#[cfg(feature = "hex")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "hex")))]
impl From<faster_hex::Error> for Error {
    fn from(value: faster_hex::Error) -> Self {
        Self::Hex(value)
    }
}

#[cfg(feature = "proof")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "proof")))]
impl From<blst::BLST_ERROR> for Error {
    fn from(e: blst::BLST_ERROR) -> Self {
        Self::Bls(e)
    }
}

#[cfg(feature = "proof")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "proof")))]
impl From<cita_trie::TrieError> for Error {
    fn from(e: cita_trie::TrieError) -> Self {
        Self::Trie(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidProofBlockHash => write!(f, "Invalid proof block hash"),
            Error::NotEnoughSignatures => write!(f, "Not enough signatures"),
            Error::VerifyMptProof => write!(f, "Verify mpt proof"),
            Error::HexPrefix => write!(f, "Hex prefix"),
            #[cfg(feature = "hex")]
            Error::Hex(e) => write!(f, "Hex error: {:?}", e),
            #[cfg(feature = "proof")]
            Error::Bls(e) => write!(f, "Bls error: {:?}", e),
            #[cfg(feature = "proof")]
            Error::Trie(e) => write!(f, "Trie error: {:?}", e),
        }
    }
}
