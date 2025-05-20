//! Demonstrate the `commonware-broadcast` primitive by sending a message from one participant to others.

mod message;

use clap::{value_parser, Arg, Command};
use commonware_broadcast::buffered::{self, Engine};
use commonware_codec::RangeCfg;
use commonware_cryptography::{Ed25519, Signer};
use commonware_p2p::authenticated::{self, Network};
use commonware_runtime::{tokio, Metrics, Runner};
use commonware_utils::{union, NZU32};
use governor::Quota;
use message::Message;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use tracing::info;

const APPLICATION_NAMESPACE: &[u8] = b"broadcast-demo";

fn main() {
    let matches = Command::new("commonware-broadcast-demo")
        .about("demonstrate commonware-broadcast")
        .arg(Arg::new("me").long("me").required(true))
        .arg(
            Arg::new("participants")
                .long("participants")
                .required(true)
                .value_delimiter(',')
                .value_parser(value_parser!(u64)),
        )
        .arg(
            Arg::new("bootstrappers")
                .long("bootstrappers")
                .required(false)
                .value_delimiter(',')
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("broadcast")
                .long("broadcast")
                .required(false)
                .help("Message to broadcast"),
        )
        .get_matches();

    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let me = matches.get_one::<String>("me").expect("identity required");
    let parts = me.split('@').collect::<Vec<_>>();
    if parts.len() != 2 {
        panic!("identity not well-formed");
    }
    let key = parts[0].parse::<u64>().expect("key not well-formed");
    let port = parts[1].parse::<u16>().expect("port not well-formed");
    let signer = Ed25519::from_seed(key);
    info!(key = ?signer.public_key(), "loaded signer");

    let participants = matches
        .get_many::<u64>("participants")
        .expect("provide participants")
        .map(|p| Ed25519::from_seed(*p).public_key())
        .collect::<Vec<_>>();

    let mut bootstrappers = Vec::new();
    if let Some(list) = matches.get_many::<String>("bootstrappers") {
        for b in list {
            let parts = b.split('@').collect::<Vec<_>>();
            let k = parts[0].parse::<u64>().expect("bootstrapper key not well-formed");
            let addr = SocketAddr::from_str(parts[1]).expect("bootstrapper addr not well-formed");
            bootstrappers.push((Ed25519::from_seed(k).public_key(), addr));
        }
    }

    let maybe_broadcast = matches.get_one::<String>("broadcast").cloned();

    let executor = tokio::Runner::default();
    executor.start(|context| async move {
        let p2p_cfg = authenticated::Config::aggressive(
            signer.clone(),
            &union(APPLICATION_NAMESPACE, b"_P2P"),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
            bootstrappers.clone(),
            1024 * 1024,
        );

        let (mut network, mut oracle) = Network::new(context.with_label("network"), p2p_cfg);

        oracle.register(0, participants.clone()).await;

        let (sender, receiver) = network.register(
            0,
            Quota::per_second(NZU32!(10)),
            256,
            Some(3),
        );

        let cfg = buffered::Config {
            public_key: signer.public_key(),
            mailbox_size: 1024,
            deque_size: 10,
            priority: false,
            codec_config: RangeCfg::from(0..=1024usize),
        };
        let (engine, mut mailbox) = Engine::new(context.with_label("broadcast"), cfg);

        network.start();
        engine.start((sender, receiver));

        if let Some(msg) = maybe_broadcast.clone() {
            let message = Message::new(0, msg.into_bytes());
            mailbox.broadcast(commonware_p2p::Recipients::All, message)
                .await
                .await
                .ok();
            info!("broadcast sent");
        }

        let commitment = Message::commitment_for_id(0);
        let received = mailbox.subscribe(None, commitment, None).await.await.unwrap();
        println!("Received: {}", String::from_utf8_lossy(&received.data));

        context.sleep(std::time::Duration::from_secs(1)).await;
    });
}
