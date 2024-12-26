use std::path::PathBuf;
use clap::Parser;
use serde::{Deserialize, Serialize};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use isok_broker::{run, Config, Error};

#[derive(Parser, Serialize, Deserialize, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {
    /// Path to your configuration file to load
    ///
    /// By default, it will look for a file named `broker.yaml` in the current directory,
    /// it will then look for it into `/etc/isok/broker.yaml`. If none of these files are found,
    /// it will use the default configuration. See [CliArgs::get_possible_paths] for more information.
    #[arg(short, long, env = "ISOK_BROKER_CONFIG_PATH")]
    config: Option<PathBuf>,
}

impl CliArgs {
    fn get_possible_paths(bin_name: &str) -> Vec<PathBuf> {
        vec![
            PathBuf::from(format!("/etc/{bin_name}/broker.yaml")),
            PathBuf::from(format!("./{bin_name}/broker.yaml")),
            PathBuf::from("./broker.yaml"),
        ]
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env_lossy())
        .init();
    let mut args = CliArgs::parse();
    if args.config.is_none() {
        let possible_paths = CliArgs::get_possible_paths(env!("CARGO_PKG_NAME"));
        tracing::debug!("No config file provided, looking for one in {:?}", possible_paths);
        for path in possible_paths {
            if path.exists() {
                tracing::info!("Using config file at {}", path.display());
                args.config = Some(path);
                break;
            }
        }
    }
    let config = match args.config {
        Some(path) => Config::from_config_file(path)?,
        None => Config::default(),
    };
    run(config).await
}