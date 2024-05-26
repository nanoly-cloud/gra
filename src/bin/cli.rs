use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use futures_timer::Delay;
use libp2p::{multiaddr::Protocol, Multiaddr};
use std::{net::Ipv4Addr, path::PathBuf};
use tokio::task;
use tracing::{debug, info, trace};
#[cfg(feature = "tracing-forest")]
use tracing_forest::ForestLayer;
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

use gra::{
    common::generate_identity,
    daemon::Daemon,
    hash::{Hash, HashOpts},
    models::Models,
    node::{Client, Node},
    reader,
    storage::Tier,
};

#[cfg(not(feature = "tracing-forest"))]
use tracing_subscriber::fmt::format::FmtSpan;

pub const NUM_BUFFERS: usize = std::mem::size_of::<usize>(); // * 8 / BLOCK_SIZE;
pub const BRANCHING_FACTOR: usize = 2;

// const BOOTSTRAP_NODES: [&str; 1] = ["/dnsaddr/nanoly.cloud"];
/// This is seed=1
const BOOTSTRAP_NODES: [&str; 1] = ["12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"];

fn init_tracing() {
    let registry = tracing_subscriber::registry::Registry::default()
        .with(EnvFilter::from_env("GRA_LOG"))
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_span_events(FmtSpan::CLOSE)
                .with_target(true)
                .with_thread_names(false)
                .with_timer(tracing_subscriber::fmt::time::SystemTime)
                .with_line_number(true)
                .with_level(true)
                .compact(),
            //
            // .json(),
            // ??? .with_ansi(true)
        );

    #[cfg(feature = "tracing-forest")]
    registry.with(ForestLayer::default()).init();

    #[cfg(not(feature = "tracing-forest"))]
    {
        let format = tracing_subscriber::fmt::layer()
            .pretty()
            .with_level(false)
            .with_target(false)
            .with_thread_names(false)
            .with_span_events(FmtSpan::CLOSE)
            .with_ansi(true)
            .with_timer(tracing_subscriber::fmt::time::SystemTime)
            .compact();

        registry.with(format).init();
    }

    trace!("Initialised Tracing");
}

const PARSER_TEMPLATE: &str = "\
        {all-args}
";

const APPLET_TEMPLATE: &str = "\
    {about-with-newline}\n\
    {usage-heading}\n    {usage}\n\
    \n\
    {all-args}{after-help}\
";

#[derive(Parser)]
#[command(name = "GRA cli", version, author, about, help_template = PARSER_TEMPLATE)]
struct Cli {
    #[arg(long, default_value = "/ip4/0.0.0.0")]
    address: Option<Multiaddr>,

    #[arg(long, default_value = "/ip4/127.0.0.1/udp/58008/quic-v1")]
    daemon_address: Multiaddr,

    #[arg(long)]
    seed: Vec<u8>,

    #[command(subcommand)]
    command: Option<Commands>,

    input: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a file to the hashmap
    Add {
        /// The path to add
        path: PathBuf,
        /// The scope to use for the hash,
        scope: Option<String>,
    },
    /// Execute a data stream
    Run {
        /// The file to run
        input: String,
    },
    /// Query a hash
    Query {
        /// The file to query
        input: String,
    },
    /// Pay respect. Mark and share the file
    F {
        input: String,
    },
    /// Show the status of the node
    Status,
    /// Configure the system
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    Dial {
        /// The address to dial
        address: Multiaddr,
    },
    // TODO: Remove this command
    /// Start as a daemon
    Daemon {},
    Listen {},
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a configuration parameter
    Set {
        /// The configuration key
        key: String,
        /// The configuration value
        value: String,
    },
    /// Get a configuration parameter
    Get {
        /// The configuration key
        key: String,
    },
}

///
/// Alright, buddy. You're not my buddy, pal.
///
/// GRA.
///
#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let opts = Cli::parse();

    // strip out usage

    let address = opts.address.unwrap_or_else(|| {
        Multiaddr::empty()
            .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(0))
    });
    info!("Address: {:?}", address);

    let daemon_address = Multiaddr::from(opts.daemon_address);
    info!("Daemon Address: {:?}", daemon_address);

    let models = Models::new(Some(vec![Tier::Memory]));

    let mut seed = opts.seed.clone();
    let seed = seed.as_mut_slice();

    let identity = generate_identity(Some(seed));

    if let Some(Commands::Daemon {}) = &opts.command {
        info!("Starting Daemon");
        let daemon = Daemon::new(address.to_owned(), identity)?;
        return Ok(daemon.run().await);
    };
    info!("Starting Node");
    let mut node = Node::new(
        address.to_owned(),
        identity,
        Some(daemon_address.to_owned()),
        Some(BOOTSTRAP_NODES),
    )?;

    let mut client = node.client();

    let handle = task::spawn(async move { node.run().await });

    /// Wait for the node to start, this is a hack to help me debug libp2p startup
    Delay::new(std::time::Duration::from_secs(5)).await;
    let Cli { command, input, .. } = opts;
    let _ = command_handler(&mut client, command, input, address).await;

    handle.await?;
    Ok(())
}

async fn command_handler(
    client: &mut Client,
    command: Option<Commands>,
    input: Option<String>,
    address: Multiaddr,
) -> Result<()> {
    match command {
        Some(Commands::Add { path, scope }) => {
            debug!("Adding {:?}", path);
            let scope = scope.map(|scope| Hash::new(scope.as_bytes(), None));
            let hash = Hash::new(
                path.to_string_lossy().as_bytes(),
                Some(HashOpts {
                    key: scope.to_owned(),
                }),
            );

            trace!("Path hash: {:?}", hash);
            reader::add_path(&path, scope.to_owned())?;

            client.start_providing(hash).await;

            Ok(())
        }
        Some(Commands::Query { input }) => {
            debug!("Querying for {:?}", input);
            let hash = Hash::new(input.as_bytes(), None);
            trace!("Hash: {:?}", hash);
            let block = client.request_block(hash, None).await?;
            debug!("Result {:?}", block);
            Ok(())
        }
        Some(Commands::Status) => todo!(),
        Some(Commands::Config { action }) => match action {
            ConfigAction::Set { key, value } => todo!(),
            ConfigAction::Get { key } => todo!(),
        },
        Some(Commands::Listen {}) => {
            debug!("Listening on {:?}", address);
            client.start_listening(address).await?;
            Ok(())
        }
        Some(Commands::Daemon {}) => Ok(()),
        Some(cmd) => {
            if let Some(input) = &input {
                // let result = node.get(&Hash::new(input.as_bytes(), None))?;
                // info!("Result: {:?}", result);
                // if let Some(result) = result {
                // }
                todo!()
            } else {
                Cli::command().print_help().map_err(|e| e.into())
            }
        }
        None => Cli::command().print_help().map_err(|e| e.into()),
    }
}
