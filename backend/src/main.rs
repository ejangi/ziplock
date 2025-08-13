//! ZipLock Backend Daemon
//!
//! This is the main entry point for the ZipLock backend service, a secure daemon
//! that manages encrypted 7z archives containing password credentials.
//!
//! The daemon provides:
//! - Secure master key management in memory
//! - AES-256 encryption/decryption of credential archives
//! - IPC API for frontend clients via Unix domain sockets
//! - File locking to prevent corruption during sync operations

use anyhow::{Context, Result};
use clap::Parser;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod config;
mod error;
mod ipc;
mod storage;

use config::Config;
use ipc::IpcServer;
use storage::ArchiveManager;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Socket path for IPC communication
    #[arg(short, long)]
    socket: Option<PathBuf>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Run in foreground (don't daemonize)
    #[arg(short, long)]
    foreground: bool,
}

/// Main daemon state
pub struct Daemon {
    config: Config,
    archive_manager: Arc<ArchiveManager>,
    ipc_server: IpcServer,
}

impl Daemon {
    /// Create a new daemon instance
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing ZipLock backend daemon");

        // Initialize archive manager
        let archive_manager = Arc::new(ArchiveManager::new(config.storage.clone())?);
        info!("Archive manager initialized");

        // Initialize IPC server
        let ipc_server = IpcServer::new(
            config.ipc.socket_path.clone(),
            Arc::clone(&archive_manager),
            config.clone(),
        )
        .await
        .context("Failed to initialize IPC server")?;
        info!("IPC server initialized on {:?}", config.ipc.socket_path);

        Ok(Self {
            config,
            archive_manager,
            ipc_server,
        })
    }

    /// Run the daemon
    pub async fn run(self) -> Result<()> {
        info!("Starting ZipLock backend daemon");

        // Start the IPC server
        let ipc_handle = tokio::spawn(async move {
            if let Err(e) = self.ipc_server.run().await {
                error!("IPC server error: {}", e);
            }
        });

        // Set up signal handling for graceful shutdown
        let shutdown_signal = async {
            let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to register SIGTERM handler");
            let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
                .expect("Failed to register SIGINT handler");

            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, initiating graceful shutdown");
                },
                _ = sigint.recv() => {
                    info!("Received SIGINT, initiating graceful shutdown");
                },
            }
        };

        // Wait for shutdown signal
        shutdown_signal.await;

        // Graceful shutdown
        info!("Shutting down ZipLock backend daemon");

        // Cancel IPC server
        ipc_handle.abort();

        // Archive manager will handle cleanup automatically
        info!("ZipLock backend daemon stopped");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_level(true),
        )
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            log_level,
        ))
        .init();

    info!("Starting ZipLock Backend v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config_path = args
        .config
        .or_else(|| {
            dirs::config_dir().map(|mut p| {
                p.push("ziplock");
                p.push("backend.yml");
                p
            })
        })
        .unwrap_or_else(|| PathBuf::from("/etc/ziplock/backend.yml"));

    let mut config = Config::load(&config_path).unwrap_or_else(|e| {
        warn!(
            "Failed to load config from {:?}: {}. Using defaults.",
            config_path, e
        );
        Config::default()
    });

    // Override config with command line arguments
    if let Some(socket) = args.socket {
        config.ipc.socket_path = socket;
    }

    // Validate configuration
    config
        .validate()
        .context("Configuration validation failed")?;
    info!("Configuration loaded and validated");

    // Create daemon
    let daemon = Daemon::new(config)
        .await
        .context("Failed to create daemon")?;

    // Run daemon
    match daemon.run().await {
        Ok(()) => {
            info!("Daemon exited successfully");
            Ok(())
        }
        Err(e) => {
            error!("Daemon failed: {}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_daemon_creation() {
        let temp_dir = tempdir().unwrap();
        let socket_path = temp_dir.path().join("test.sock");

        let config = Config {
            ipc: config::IpcConfig {
                socket_path,
                ..Default::default()
            },
            ..Default::default()
        };

        let daemon = Daemon::new(config).await;
        assert!(daemon.is_ok());
    }

    #[test]
    fn test_args_parsing() {
        use clap::Parser;

        let args = Args::try_parse_from([
            "ziplock-backend",
            "--debug",
            "--foreground",
            "--socket",
            "/tmp/test.sock",
        ]);

        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.debug);
        assert!(args.foreground);
        assert_eq!(args.socket, Some(PathBuf::from("/tmp/test.sock")));
    }
}
