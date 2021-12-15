use std::time::Duration;
use tentacle::{bytes::Bytes, service::ServiceAsyncControl, SessionId};

use async_trait::async_trait;
use protocol::traits::{Context, MessageCodec, Priority, Rpc};
use protocol::{tokio, ProtocolResult};

use crate::endpoint::Endpoint;
use crate::error::{ErrorKind, NetworkError};
use crate::message::{Headers, NetworkMessage};
use crate::reactor::MessageRouter;
use crate::rpc::RpcResponse;
use crate::traits::NetworkContext;

#[derive(Clone)]
pub struct NetworkRpc {
    transmitter:       ServiceAsyncControl,
    pub(crate) router: MessageRouter,
}

impl NetworkRpc {
    pub fn new(transmitter: ServiceAsyncControl, router: MessageRouter) -> Self {
        NetworkRpc {
            transmitter,
            router,
        }
    }

    async fn send(
        &self,
        _ctx: Context,
        session_id: SessionId,
        data: Bytes,
        priority: Priority,
    ) -> Result<(), NetworkError> {
        match priority {
            Priority::Normal => self
                .transmitter
                .clone()
                .send_message_to(
                    session_id,
                    crate::protocols::SupportProtocols::Transmitter.protocol_id(),
                    data,
                )
                .await
                .unwrap(),
            Priority::High => self
                .transmitter
                .clone()
                .send_message_to(
                    session_id,
                    crate::protocols::SupportProtocols::Transmitter.protocol_id(),
                    data,
                )
                .await
                .unwrap(),
        }
        Ok(())
    }
}

#[async_trait]
impl Rpc for NetworkRpc {
    async fn call<M, R>(
        &self,
        mut cx: Context,
        endpoint: &str,
        mut msg: M,
        priority: Priority,
    ) -> ProtocolResult<R>
    where
        M: MessageCodec,
        R: MessageCodec,
    {
        let endpoint = endpoint.parse::<Endpoint>()?;
        let sid = cx.session_id()?;
        let rpc_map = &self.router.rpc_map;
        let rid = rpc_map.next_rpc_id();
        let connected_addr = cx.remote_connected_addr();
        let done_rx = rpc_map.insert::<RpcResponse>(sid, rid);

        struct _Guard {
            transmitter: MessageRouter,
            sid:         SessionId,
            rid:         u64,
        }

        impl Drop for _Guard {
            fn drop(&mut self) {
                // Simple take then drop if there is one
                let rpc_map = &self.transmitter.rpc_map;
                let _ = rpc_map.take::<RpcResponse>(self.sid, self.rid);
            }
        }

        let _guard = _Guard {
            transmitter: self.router.clone(),
            sid,
            rid,
        };

        let data = msg.encode_msg()?;
        let endpoint = endpoint.extend(&rid.to_string())?;
        let headers = Headers::default();
        // if let Some(state) = common_apm::muta_apm::MutaTracer::span_state(&cx) {
        //     headers.set_trace_id(state.trace_id());
        //     headers.set_span_id(state.span_id());
        //     log::info!("no trace id found for rpc {}", endpoint.full_url());
        // }
        // common_apm::metrics::network::on_network_message_sent(endpoint.full_url());

        let ctx = cx.set_url(endpoint.root());
        let net_msg = NetworkMessage::new(endpoint, data, headers).encode()?;

        self.send(ctx, sid, net_msg, priority).await?;

        let timeout = tokio::time::timeout(Duration::from_secs(10), done_rx);
        match timeout.await {
            Ok(Ok(ret)) => match ret {
                RpcResponse::Success(v) => {
                    // common_apm::metrics::network::NETWORK_RPC_RESULT_COUNT_VEC_STATIC
                    //     .success
                    //     .inc();
                    // common_apm::metrics::network::NETWORK_PROTOCOL_TIME_HISTOGRAM_VEC_STATIC
                    //     .rpc
                    //     .observe(common_apm::metrics::duration_to_sec(inst.elapsed()));

                    Ok(R::decode_msg(v)?)
                }
                RpcResponse::Error(e) => Err(NetworkError::RemoteResponse(e).into()),
            },
            Ok(Err(_)) => Err(NetworkError::from(ErrorKind::RpcDropped(connected_addr)).into()),
            Err(_) => {
                log::info!("rpc call timeout");
                Err(NetworkError::from(ErrorKind::RpcTimeout(connected_addr)).into())
            }
        }
    }

    async fn response<M>(
        &self,
        mut cx: Context,
        endpoint: &str,
        ret: ProtocolResult<M>,
        priority: Priority,
    ) -> ProtocolResult<()>
    where
        M: MessageCodec,
    {
        let endpoint = endpoint.parse::<Endpoint>()?;
        let sid = cx.session_id()?;
        let rid = cx.rpc_id()?;
        let resp = match ret.map_err(|e| e.to_string()) {
            Ok(mut m) => RpcResponse::Success(m.encode_msg()?),
            Err(err_msg) => RpcResponse::Error(err_msg),
        };

        let encoded_resp = resp.encode();
        let endpoint = endpoint.extend(&rid.to_string())?;
        let headers = Headers::default();
        // if let Some(state) = common_apm::muta_apm::MutaTracer::span_state(&cx) {
        //     headers.set_trace_id(state.trace_id());
        //     headers.set_span_id(state.span_id());
        //     log::info!("no trace id found for rpc {}", endpoint.full_url());
        // }
        // common_apm::metrics::network::on_network_message_sent(endpoint.full_url());

        let ctx = cx.set_url(endpoint.root());
        let net_msg = NetworkMessage::new(endpoint, encoded_resp, headers).encode()?;

        self.send(ctx, sid, net_msg, priority).await?;

        Ok(())
    }
}
