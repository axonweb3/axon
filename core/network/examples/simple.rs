use core_network::{NetworkConfig, NetworkService, NetworkServiceHandle};
use protocol::{
    async_trait, tokio,
    traits::{Context, Gossip, MessageHandler, Priority, Rpc, TrustFeedback},
    types::Bytes,
    ProtocolError,
};

use std::time::Duration;
use tentacle::secio::SecioKeyPair;

const RELEASE_CHANNEL: &str = "/gossip/cprd/cyperpunk7702_released";
const SHOP_CASH_CHANNEL: &str = "/rpc_call/v3/steam";
const SHOP_CHANNEL: &str = "/rpc_resp/v3/steam";

struct TakeMyMoney {
    shop: NetworkServiceHandle,
}

#[async_trait]
impl MessageHandler for TakeMyMoney {
    type Message = Bytes;

    async fn process(&self, ctx: Context, msg: Self::Message) -> TrustFeedback {
        let sell = async move {
            log::info!("Rush to {:?}. Shut up, take my money", msg);

            let copy: Bytes = self
                .shop
                .call(ctx, SHOP_CASH_CHANNEL, Bytes::from("2345"), Priority::High)
                .await?;
            log::info!("Got my copy {:?}", copy);

            Ok::<(), ProtocolError>(())
        };
        match sell.await {
            Ok(_) => TrustFeedback::Good,
            Err(e) => {
                log::error!("sell {}", e);
                TrustFeedback::Bad("sell failed".to_owned())
            }
        }
    }
}

struct Checkout {
    dealer: NetworkServiceHandle,
}

#[async_trait]
impl MessageHandler for Checkout {
    type Message = Bytes;

    async fn process(&self, ctx: Context, _msg: Self::Message) -> TrustFeedback {
        match self
            .dealer
            .response(ctx, SHOP_CHANNEL, Ok(Bytes::from("1234")), Priority::High)
            .await
        {
            Ok(_) => TrustFeedback::Good,
            Err(e) => TrustFeedback::Bad(format!("send copy {}", e)),
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let bt_seckey = [8u8; 32];
    let bt_keypair = SecioKeyPair::secp256k1_raw_key(bt_seckey).expect("keypair");
    let peer_id = bt_keypair.peer_id().to_base58();

    if std::env::args().nth(1) == Some("server".to_string()) {
        log::info!("Starting server");

        let bt_conf = NetworkConfig::new()
            .listen_addr("/ip4/127.0.0.1/tcp/1337".parse().unwrap())
            .secio_keypair(&bt_seckey)
            .unwrap();
        let mut bootstrap = NetworkService::new(bt_conf, bt_keypair);
        let handle = bootstrap.handle();

        let check_out =
            Checkout {
                dealer: handle.clone(),
            };

        bootstrap
            .register_endpoint_handler(SHOP_CASH_CHANNEL, check_out)
            .unwrap();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;

                let ctx = Context::default();
                handle
                    .broadcast(
                        ctx.clone(),
                        RELEASE_CHANNEL,
                        Bytes::from(""),
                        Priority::High,
                    )
                    .await
                    .unwrap();
            }
        });

        bootstrap.run().await
    } else {
        log::info!("Starting client");

        let peer_conf = NetworkConfig::new()
            .listen_addr("/ip4/127.0.0.1/tcp/1338".parse().unwrap())
            .bootstraps(vec![format!("/ip4/127.0.0.1/tcp/1337/p2p/{}", peer_id)
                .parse()
                .unwrap()]);

        let mut peer = NetworkService::new(peer_conf, bt_keypair);
        let handle = peer.handle();

        let take_my_money = TakeMyMoney { shop: handle };

        peer.register_endpoint_handler(RELEASE_CHANNEL, take_my_money)
            .unwrap();
        peer.register_rpc_response(SHOP_CHANNEL).unwrap();

        peer.run().await;
    }
}
