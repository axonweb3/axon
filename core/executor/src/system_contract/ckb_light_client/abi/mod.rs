pub mod ckb_light_client_abi;

use ckb_types::{packed, prelude::*};

impl From<ckb_light_client_abi::Header> for packed::Header {
    fn from(header: ckb_light_client_abi::Header) -> packed::Header {
        let raw = packed::RawHeader::new_builder()
            .compact_target(header.compact_target.pack())
            .dao(header.dao.pack())
            .epoch(header.epoch.pack())
            .extra_hash(header.block_hash.pack())
            .number(header.number.pack())
            .parent_hash(header.parent_hash.pack())
            .proposals_hash(header.proposals_hash.pack())
            .timestamp(header.timestamp.pack())
            .transactions_root(header.transactions_root.pack())
            .version(header.version.pack())
            .build();

        packed::Header::new_builder()
            .raw(raw)
            .nonce(header.nonce.pack())
            .build()
    }
}

impl From<packed::Header> for ckb_light_client_abi::Header {
    fn from(header: packed::Header) -> Self {
        let raw = header.raw();
        ckb_light_client_abi::Header {
            compact_target:    raw.compact_target().unpack(),
            dao:               raw.dao().unpack().into(),
            epoch:             raw.epoch().unpack(),
            block_hash:        raw.extra_hash().unpack().into(),
            number:            raw.number().unpack(),
            parent_hash:       raw.parent_hash().unpack().into(),
            proposals_hash:    raw.proposals_hash().unpack().into(),
            timestamp:         raw.timestamp().unpack(),
            transactions_root: raw.transactions_root().unpack().into(),
            version:           raw.version().unpack(),
            nonce:             header.nonce().unpack(),
            uncles_hash:       [0u8; 32],
        }
    }
}
