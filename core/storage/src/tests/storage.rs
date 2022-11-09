use std::sync::Arc;

use protocol::traits::{CommonStorage, Context, Storage};
use protocol::types::Hasher;

use crate::adapter::memory::MemoryAdapter;
use crate::tests::{get_random_bytes, mock_block, mock_proof, mock_receipt, mock_signed_tx};
use crate::ImplStorage;

macro_rules! exec {
    ($func: expr) => {
        futures::executor::block_on(async { $func.await.unwrap() })
    };
}

#[test]
fn test_storage_block_insert() {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);

    let height = 100;
    let block = mock_block(height, Hasher::digest(get_random_bytes(10)));
    let block_hash = block.hash();

    exec!(storage.insert_block(Context::new(), block));

    let block = exec!(storage.get_latest_block(Context::new()));
    assert_eq!(height, block.header.number);

    let block = exec!(storage.get_block(Context::new(), height));
    assert_eq!(Some(height), block.map(|b| b.header.number));

    let block = exec!(storage.get_block_by_hash(Context::new(), &block_hash));
    assert_eq!(height, block.unwrap().header.number);
}

#[test]
fn test_storage_receipts_insert() {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let height = 2077;

    let mut receipts = Vec::new();
    let mut hashes = Vec::new();

    for _ in 0..1 {
        let hash = Hasher::digest(get_random_bytes(10));
        hashes.push(hash);
        let receipt = mock_receipt(hash);
        receipts.push(receipt);
    }

    exec!(storage.insert_receipts(Context::new(), height, receipts.clone()));
    let receipts_2 = exec!(storage.get_receipts(Context::new(), height, &hashes));

    for i in 0..1 {
        assert_eq!(
            Some(receipts.get(i).unwrap()),
            receipts_2.get(i).unwrap().as_ref()
        );
    }
}

#[test]
fn test_storage_transactions_insert() {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let height = 2020;

    let mut transactions = Vec::new();
    let mut hashes = Vec::new();

    for _ in 0..10 {
        let transaction = mock_signed_tx();
        hashes.push(transaction.transaction.hash);
        transactions.push(transaction);
    }

    exec!(storage.insert_transactions(Context::new(), height, transactions.clone()));
    let transactions_2 = exec!(storage.get_transactions(Context::new(), height, &hashes));

    for i in 0..10 {
        assert_eq!(
            Some(transactions.get(i).unwrap()),
            transactions_2.get(i).unwrap().as_ref()
        );
    }
}

#[test]
fn test_storage_latest_proof_insert() {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);

    let block_hash = Hasher::digest(get_random_bytes(10));
    let proof = mock_proof(block_hash);

    exec!(storage.update_latest_proof(Context::new(), proof.clone()));
    let proof_2 = exec!(storage.get_latest_proof(Context::new(),));

    assert_eq!(proof.block_hash, proof_2.block_hash);
}

#[test]
fn test_storage_evm_code_insert() {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);

    let code = get_random_bytes(1000);
    let code_hash = Hasher::digest(&code);
    let address = Hasher::digest(code_hash);

    exec!(storage.insert_code(Context::new(), address, code_hash, code.clone()));

    let code_2 = exec!(storage.get_code_by_hash(Context::new(), &code_hash));
    assert_eq!(code, code_2.unwrap());

    let code_3 = exec!(storage.get_code_by_address(Context::new(), &address));
    assert_eq!(code, code_3.unwrap());
}

#[test]
#[cfg(feature = "ibc")]
fn test_ibc_storage() {
    test_ibc_get_set_client_type();
    test_ibc_get_set_client_state();
    test_ibc_get_set_consensus_state();
    test_ibc_get_set_connection_end();
    test_ibc_get_set_connection_to_client();
    test_ibc_get_set_channel();
    test_ibc_get_set_packet_commitment();
    test_ibc_get_set_packet_receipt();
    test_ibc_get_set_packet_acknowledgement();
    test_ibc_get_set_next_sequence_send();
    test_ibc_get_set_next_sequence_recv();
    test_ibc_get_set_next_sequence_ack();
    test_ibc_get_keys_by_path();
}

#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_set_client_type() {
    use cosmos_ibc::core::ics02_client::client_type::ClientType;
    use cosmos_ibc::core::ics24_host::identifier::ClientId;
    use protocol::traits::IbcCrossChainStorage;
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let client_id = ClientId::new(ClientType::Tendermint, 1033).unwrap();
    let set_result = storage.set_client_type(client_id.clone(), ClientType::Tendermint);
    assert!(set_result.is_ok());

    let get_result = storage.get_client_type(&client_id);
    assert!(set_result.is_ok());

    let result_client_type = get_result.unwrap_or_default().unwrap();
    assert_eq!(ClientType::Tendermint, result_client_type);
}

#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_set_client_state() {
    use cosmos_ibc::core::ics02_client::client_type::ClientType;
    use cosmos_ibc::core::ics24_host::identifier::ClientId;
    use cosmos_ibc::mock::client_state::MockClientState;
    use cosmos_ibc::mock::header::MockHeader;
    use protocol::traits::IbcCrossChainStorage;
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let client_id = ClientId::new(ClientType::Tendermint, 1033).unwrap();
    use cosmos_ibc::core::ics02_client::client_state::AnyClientState;
    let mock_client_state = AnyClientState::Mock(MockClientState::new(MockHeader::default()));
    let set_result = storage.set_client_state(client_id.clone(), mock_client_state.clone());
    assert!(set_result.is_ok());

    let get_result = storage.get_client_state(&client_id);
    assert!(get_result.is_ok());

    assert_eq!(get_result.unwrap().unwrap(), mock_client_state);
}
#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_set_consensus_state() {
    use cosmos_ibc::core::ics02_client::client_consensus::AnyConsensusState;
    use cosmos_ibc::core::ics02_client::client_type::ClientType;
    use cosmos_ibc::core::ics24_host::identifier::ClientId;
    use cosmos_ibc::mock::client_state::MockConsensusState;
    use cosmos_ibc::mock::header::MockHeader;
    use cosmos_ibc::Height;
    use protocol::traits::IbcCrossChainStorage;

    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let client_id = ClientId::new(ClientType::Tendermint, 1033).unwrap();
    let mock_consensus_state =
        AnyConsensusState::Mock(MockConsensusState::new(MockHeader::default()));
    let height = Height::new(0, 1).unwrap();
    let set_result =
        storage.set_consensus_state(client_id.clone(), height, mock_consensus_state.clone());
    assert!(set_result.is_ok());

    let get_ret = storage.get_consensus_state(
        &client_id,
        height.revision_number(),
        height.revision_height(),
    );
    assert!(get_ret.is_ok());

    assert_eq!(mock_consensus_state, get_ret.unwrap().unwrap());
}
#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_set_connection_end() {
    use cosmos_ibc::core::ics03_connection::connection::ConnectionEnd;
    use cosmos_ibc::core::ics24_host::identifier::ConnectionId;
    use protocol::traits::IbcCrossChainStorage;
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let connection_id = ConnectionId::new(11);
    let connection_end = ConnectionEnd::default();
    let set_ret = storage.set_connection_end(connection_id.clone(), connection_end.clone());
    assert!(set_ret.is_ok());

    let get_ret = storage.get_connection_end(&connection_id);
    assert!(get_ret.is_ok());

    assert_eq!(connection_end, get_ret.unwrap().unwrap());
}
#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_set_connection_to_client() {
    use cosmos_ibc::core::ics02_client::client_type::ClientType;
    use cosmos_ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
    use protocol::traits::IbcCrossChainStorage;
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let connect_id = ConnectionId::new(3);
    let connect_id_err = ConnectionId::new(4);
    let client_id = ClientId::new(ClientType::Tendermint, 3).unwrap();
    let set_result = storage.set_connection_to_client(connect_id.clone(), &client_id);
    assert!(set_result.is_ok());

    let get_result = storage.get_connection_to_client(&client_id);
    assert!(get_result.is_ok());

    let result_client_id = get_result.unwrap_or_default().unwrap();
    assert_eq!(connect_id, result_client_id[0]);
    assert_ne!(connect_id_err, result_client_id[0]);
}
#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_set_channel() {
    use cosmos_ibc::core::ics04_channel::channel::ChannelEnd;
    use cosmos_ibc::core::ics24_host::identifier::{ChannelId, PortId};
    use protocol::traits::IbcCrossChainStorage;
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let channel_end = ChannelEnd::default();
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let set_ret = storage.set_channel(port_id.clone(), channel_id.clone(), channel_end.clone());
    assert!(set_ret.is_ok());

    let get_ret = storage.get_channel_end(&(port_id, channel_id));
    assert!(get_ret.is_ok());

    assert_eq!(channel_end, get_ret.unwrap().unwrap());
}
#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_set_packet_commitment() {
    use cosmos_ibc::core::{
        ics04_channel::{commitment::PacketCommitment, packet::Sequence},
        ics24_host::identifier::{ChannelId, PortId},
    };
    use protocol::traits::IbcCrossChainStorage;
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let sequence = Sequence::default();
    let commitment = PacketCommitment::from(vec![1]);
    let set_ret = storage.set_packet_commitment(
        (port_id.clone(), channel_id.clone(), sequence),
        commitment.clone(),
    );
    assert!(set_ret.is_ok());

    let get_ret = storage.get_packet_commitment(&(port_id.clone(), channel_id.clone(), sequence));
    assert!(get_ret.is_ok());

    assert_eq!(commitment, get_ret.unwrap().unwrap());

    let del_ret = storage.delete_packet_commitment((port_id.clone(), channel_id.clone(), sequence));
    assert!(del_ret.is_ok());

    let get_ret2 = storage.get_packet_commitment(&(port_id, channel_id, sequence));
    assert!(get_ret2.unwrap().is_none());
}
#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_set_packet_receipt() {
    use cosmos_ibc::core::ics04_channel::packet::{Receipt, Sequence};
    use cosmos_ibc::core::ics24_host::identifier::{ChannelId, PortId};
    use protocol::traits::IbcCrossChainStorage;
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let sequence = Sequence::default();
    let receipt = Receipt::Ok;
    let set_ret =
        storage.set_packet_receipt((port_id.clone(), channel_id.clone(), sequence), receipt);
    assert!(set_ret.is_ok());

    let get_ret = storage.get_packet_receipt(&(port_id, channel_id, sequence));
    assert!(get_ret.is_ok());
}
#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_set_packet_acknowledgement() {
    use cosmos_ibc::core::ics04_channel::commitment::AcknowledgementCommitment;
    use cosmos_ibc::core::ics04_channel::packet::Sequence;
    use cosmos_ibc::core::ics24_host::identifier::{ChannelId, PortId};
    use protocol::traits::IbcCrossChainStorage;
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let sequence = Sequence::default();
    let ack_commitment = AcknowledgementCommitment::from(vec![1]);
    let set_ret = storage.set_packet_acknowledgement(
        (port_id.clone(), channel_id.clone(), sequence),
        ack_commitment.clone(),
    );
    assert!(set_ret.is_ok());

    let get_ret =
        storage.get_packet_acknowledgement(&(port_id.clone(), channel_id.clone(), sequence));
    assert!(get_ret.is_ok());

    assert_eq!(ack_commitment, get_ret.unwrap().unwrap());

    let del_ret =
        storage.delete_packet_acknowledgement((port_id.clone(), channel_id.clone(), sequence));
    assert!(del_ret.is_ok());

    let get_ret2 = storage.get_packet_acknowledgement(&(port_id, channel_id, sequence));
    assert!(get_ret2.unwrap().is_none());
}
#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_set_next_sequence_send() {
    use cosmos_ibc::core::ics04_channel::packet::Sequence;
    use cosmos_ibc::core::ics24_host::identifier::{ChannelId, PortId};
    use protocol::traits::IbcCrossChainStorage;
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let sequence = Sequence::default();
    let set_ret = storage.set_next_sequence_send(port_id.clone(), channel_id.clone(), sequence);
    assert!(set_ret.is_ok());

    let get_ret = storage.get_next_sequence_send(&(port_id, channel_id));
    assert!(get_ret.is_ok());

    assert_eq!(sequence, get_ret.unwrap().unwrap());
}
#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_set_next_sequence_recv() {
    use cosmos_ibc::core::ics04_channel::packet::Sequence;
    use cosmos_ibc::core::ics24_host::identifier::{ChannelId, PortId};
    use protocol::traits::IbcCrossChainStorage;
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let sequence = Sequence::default();
    let set_ret = storage.set_next_sequence_recv(port_id.clone(), channel_id.clone(), sequence);
    assert!(set_ret.is_ok());

    let get_ret = storage.get_next_sequence_recv(&(port_id, channel_id));
    assert!(get_ret.is_ok());

    assert_eq!(sequence, get_ret.unwrap().unwrap());
}
#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_set_next_sequence_ack() {
    use cosmos_ibc::core::ics04_channel::packet::Sequence;
    use cosmos_ibc::core::ics24_host::identifier::{ChannelId, PortId};
    use protocol::traits::IbcCrossChainStorage;
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let sequence = Sequence::default();
    let set_ret = storage.set_next_sequence_ack(port_id.clone(), channel_id.clone(), sequence);
    assert!(set_ret.is_ok());

    let get_ret = storage.get_next_sequence_ack(&(port_id, channel_id));
    assert!(get_ret.is_ok());

    assert_eq!(sequence, get_ret.unwrap().unwrap());
}

#[test]
#[cfg(feature = "ibc")]
fn test_ibc_get_keys_by_path() {
    test_get_keys_by_path_case01();
    test_get_keys_by_path_case02();
    test_get_keys_by_path_case03();
}

#[test]
#[cfg(feature = "ibc")]
fn test_get_keys_by_path_case01() {
    use cosmos_ibc::core::ics04_channel::packet::Sequence;
    use cosmos_ibc::core::ics24_host::{
        identifier::{ChannelId, ClientId, PortId},
        path::SeqAcksPath,
    };
    use protocol::traits::IbcCrossChainStorage;
    use protocol::types::ibc::Path;

    use cosmos_ibc::core::ics02_client::client_type::ClientType;

    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let client_id = ClientId::new(ClientType::Tendermint, 1033).unwrap();
    let client_id2 = ClientId::new(ClientType::Tendermint, 22).unwrap();
    let client_id3 = ClientId::new(ClientType::Tendermint, 222).unwrap();
    let _set_result = storage.set_client_type(client_id, ClientType::Tendermint);
    let _set_result = storage.set_client_type(client_id2, ClientType::Tendermint);
    let set_result = storage.set_client_type(client_id3, ClientType::Tendermint);
    assert!(set_result.is_ok());

    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let path = SeqAcksPath(port_id.clone(), channel_id.clone());
    let local_path = Path::from(path);
    let sequence = Sequence::default();
    let set_ret = storage.set_next_sequence_ack(port_id, channel_id, sequence);
    assert!(set_ret.is_ok());
    let para1: Path = String::from("nextSequenceAck")
        .try_into()
        .expect("'channelEnds/ports' expected to be a valid Path"); // nextSequenceAck/ports
    let res = storage.get_keys_by_prefix(&para1);
    assert!(res.is_ok());
    assert!(local_path.starts_with(&para1));
    assert_eq!(res.unwrap().len(), 1);

    let para2: Path = String::from("clients")
        .try_into()
        .expect("'channelEnds/ports' expected to be a valid Path");
    let res2 = storage.get_keys_by_prefix(&para2);
    // ClientTypeSchema  SeqAcksSchema
    assert!(res2.is_ok());
    assert_eq!(res2.unwrap().len(), 3);
}

#[test]
#[cfg(feature = "ibc")]
fn test_get_keys_by_path_case02() {
    use cosmos_ibc::core::ics02_client::client_type::ClientType;
    use cosmos_ibc::core::ics04_channel::packet::Sequence;
    use cosmos_ibc::core::ics24_host::identifier::ClientId;
    use cosmos_ibc::core::ics24_host::identifier::{ChannelId, PortId};
    use protocol::traits::IbcCrossChainStorage;
    use protocol::types::ibc::Path;

    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);
    let client_id = ClientId::new(ClientType::Tendermint, 1033).unwrap();
    let client_id2 = ClientId::new(ClientType::Tendermint, 22).unwrap();
    let client_id3 = ClientId::new(ClientType::Tendermint, 222).unwrap();
    let _set_result = storage.set_client_type(client_id, ClientType::Tendermint);
    let _set_result = storage.set_client_type(client_id2, ClientType::Tendermint);
    let set_result = storage.set_client_type(client_id3, ClientType::Tendermint);
    assert!(set_result.is_ok());

    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let sequence = Sequence::default();
    let set_ret = storage.set_next_sequence_ack(port_id, channel_id, sequence);
    assert!(set_ret.is_ok());

    use cosmos_ibc::mock::client_state::MockClientState;
    use cosmos_ibc::mock::header::MockHeader;
    let client_id = ClientId::new(ClientType::Tendermint, 1033).unwrap();
    use cosmos_ibc::core::ics02_client::client_state::AnyClientState;
    let mock_client_state = AnyClientState::Mock(MockClientState::new(MockHeader::default()));
    let set_result = storage.set_client_state(client_id.clone(), mock_client_state.clone());
    assert!(set_result.is_ok());
    let get_result = storage.get_client_state(&client_id);
    assert!(get_result.is_ok());
    assert_eq!(get_result.unwrap().unwrap(), mock_client_state);

    let para1: Path = String::from("clients")
        .try_into()
        .expect("'channelEnds/ports' expected to be a valid Path"); // nextSequenceAck/ports
    let res = storage.get_keys_by_prefix(&para1);
    // ClientTypeSchema  SeqAcksSchema
    assert!(res.is_ok());
    assert_eq!(res.unwrap().len(), 4);
}

#[test]
#[cfg(feature = "ibc")]
fn test_get_keys_by_path_case03() {
    use cosmos_ibc::core::ics04_channel::packet::Sequence;
    use cosmos_ibc::core::ics24_host::{
        identifier::{ChannelId, PortId},
        path::SeqAcksPath,
    };
    use protocol::traits::IbcCrossChainStorage;
    use protocol::types::ibc::Path;

    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()), 10);

    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let path = SeqAcksPath(port_id.clone(), channel_id.clone());
    let local_path = Path::from(path);
    let sequence = Sequence::default();
    let set_ret = storage.set_next_sequence_ack(port_id, channel_id, sequence);
    assert!(set_ret.is_ok());
    let para1: Path = String::from("nextSequenceAck")
        .try_into()
        .expect("'channelEnds/ports' expected to be a valid Path");
    let res = storage.get_keys_by_prefix(&para1);
    assert!(res.is_ok());
    assert!(local_path.starts_with(&para1));
    assert_eq!(res.unwrap().len(), 1);

    let para2: Path = String::from("clients")
        .try_into()
        .expect("'channelEnds/ports' expected to be a valid Path");
    let res2 = storage.get_keys_by_prefix(&para2);
    assert!(res2.is_ok());
    assert_eq!(res2.unwrap().len(), 0);
    // *****should panic: *****
    // let para3: Path = String::from("")
    //     .try_into()
    //     .expect("'channelEnds/ports' expected to be a valid Path");
    // let res3 = storage.get_keys_by_prefix(&para3);
    // assert!(res3.is_ok());
    // assert_eq!(res3.unwrap().len(), 0);
}

#[test]
#[cfg(feature = "ibc")]
#[should_panic]
fn test_get_keys_by_path_case04() {
    use protocol::types::ibc::Path;
    let _: Path = String::from("")
        .try_into()
        .expect("'channelEnds/ports' expected to be a valid Path");
}

#[test]
#[cfg(feature = "ibc")]
fn test_mock_all_ibc_path() {
    use cosmos_ibc::{
        core::{
            ics02_client::client_type::ClientType,
            ics04_channel::packet::Sequence,
            ics24_host::{
                identifier::{ChannelId, ClientId, ConnectionId, PortId},
                path::{
                    AcksPath, ChannelEndsPath, ClientConnectionsPath, ClientConsensusStatePath,
                    ClientStatePath, ClientTypePath, CommitmentsPath, ConnectionsPath,
                    ReceiptsPath, SeqAcksPath, SeqRecvsPath, SeqSendsPath,
                },
            },
        },
        Height,
    };
    use protocol::types::ibc::{KeyPrefix, Path as LocalPath};
    // ClientTypeSchema
    let test_num = 1234567;
    let client_id = ClientId::new(ClientType::Tendermint, test_num).unwrap();
    let path = ClientTypePath(client_id);
    let local_path: LocalPath = LocalPath::try_from(path).unwrap();
    let prefix: KeyPrefix = KeyPrefix::from(&local_path);
    println!("ClientTypeSchema: {}", String::from(&prefix));
    // ClientStateSchema
    let client_id = ClientId::new(ClientType::Tendermint, test_num).unwrap();
    let path = ClientStatePath(client_id);
    let local_path: LocalPath = LocalPath::try_from(path).unwrap();
    let prefix: KeyPrefix = KeyPrefix::from(&local_path);
    println!("ClientStateSchema: {}", String::from(&prefix));
    // ClientConsensusStateSchema
    let client_id = ClientId::new(ClientType::Tendermint, test_num).unwrap();
    let height = Height::new(0, 1).unwrap();
    let client_consensus_state_path = ClientConsensusStatePath {
        client_id,
        epoch: height.revision_number(),
        height: height.revision_height(),
    };
    let local_path: LocalPath = LocalPath::try_from(client_consensus_state_path).unwrap();
    let prefix: KeyPrefix = KeyPrefix::from(&local_path);
    println!("ClientConsensusStateSchema: {}", String::from(&prefix));
    // ConnectionEndSchema
    let connection_id = ConnectionId::new(test_num);
    let path = ConnectionsPath(connection_id);
    let local_path: LocalPath = LocalPath::try_from(path).unwrap();
    let prefix: KeyPrefix = KeyPrefix::from(&local_path);
    println!("ConnectionEndSchema: {}", String::from(&prefix));
    // ConnectionIdsSchema
    let client_id = ClientId::new(ClientType::Tendermint, test_num).unwrap();
    let path = ClientConnectionsPath(client_id);
    let local_path: LocalPath = LocalPath::try_from(path).unwrap();
    let prefix: KeyPrefix = KeyPrefix::from(&local_path);
    println!("ConnectionIdsSchema: {}", String::from(&prefix));
    // ChannelEndSchema
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let path = ChannelEndsPath(port_id, channel_id);
    let local_path: LocalPath = LocalPath::try_from(path).unwrap();
    let prefix: KeyPrefix = KeyPrefix::from(&local_path);
    println!("ChannelEndSchema: {}", String::from(&prefix));
    // SeqSendsSchema
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let path = SeqSendsPath(port_id, channel_id);
    let local_path: LocalPath = LocalPath::try_from(path).unwrap();
    let prefix: KeyPrefix = KeyPrefix::from(&local_path);
    println!("SeqSendsSchema: {}", String::from(&prefix));
    // SeqRecvsSchema
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let path = SeqRecvsPath(port_id, channel_id);
    let local_path: LocalPath = LocalPath::try_from(path).unwrap();
    let prefix: KeyPrefix = KeyPrefix::from(&local_path);
    println!("SeqRecvsSchema: {}", String::from(&prefix));
    // SeqAcksSchema
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let path = SeqAcksPath(port_id, channel_id);
    let local_path: LocalPath = LocalPath::try_from(path).unwrap();
    let prefix: KeyPrefix = KeyPrefix::from(&local_path);
    println!("SeqAcksSchema: {}", String::from(&prefix));
    // PacketCommitmentSchema
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let sequence = Sequence::default();
    let path = CommitmentsPath {
        port_id,
        channel_id,
        sequence,
    };
    let local_path: LocalPath = LocalPath::try_from(path).unwrap();
    let prefix: KeyPrefix = KeyPrefix::from(&local_path);
    println!("PacketCommitmentSchema: {}", String::from(&prefix));
    // AcknowledgementCommitmentSchema
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let sequence = Sequence::default();
    let path = AcksPath {
        port_id,
        channel_id,
        sequence,
    };
    let local_path: LocalPath = LocalPath::try_from(path).unwrap();
    let prefix: KeyPrefix = KeyPrefix::from(&local_path);
    println!("AcknowledgementCommitmentSchema: {}", String::from(&prefix));
    // ReceiptSchema
    let port_id = PortId::default();
    let channel_id = ChannelId::default();
    let sequence = Sequence::default();
    let path = ReceiptsPath {
        port_id,
        channel_id,
        sequence,
    };
    let local_path: LocalPath = LocalPath::try_from(path).unwrap();
    let prefix: KeyPrefix = KeyPrefix::from(&local_path);
    println!("ReceiptSchema: {}", String::from(&prefix));
    // clients/07-tendermint-1234567/clientType
    // clients/07-tendermint-1234567/clientState
    // clients/07-tendermint-1234567/consensusStates/0-1
    // connections/connection-1234567
    // clients/07-tendermint-1234567/connections
    // channelEnds/ports/defaultPort/channels/channel-0
    // nextSequenceSend/ports/defaultPort/channels/channel-0
    // nextSequenceRecv/ports/defaultPort/channels/channel-0
    // nextSequenceAck/ports/defaultPort/channels/channel-0
    // commitments/ports/defaultPort/channels/channel-0/sequences/0
    // acks/ports/defaultPort/channels/channel-0/sequences/0
    // receipts/ports/defaultPort/channels/channel-0/sequences/0
}
