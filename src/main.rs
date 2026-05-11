use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use shs_cli::encoding::{decode_share, encode_share};
use shs_cli::shamir::{split, reconstruct};

#[derive(Parser)]
#[command(name = "shs-cli")]
#[command(about = "Shamir's Secret Sharing — split and reconstruct secrets")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Split a secret into shares
    Split {
        /// The secret string to split
        #[arg(short, long)]
        secret: String,

        /// Minimum shares needed to reconstruct
        #[arg(short, long)]
        threshold: usize,

        /// Total number of shares to generate
        #[arg(short = 'n', long)]
        shares: usize,
    },

    /// Reconstruct a secret from shares
    Reconstruct {
        /// Share strings in "index-hex" format
        #[arg(short, long, num_args = 1..)]
        shares: Vec<String>,

        /// Threshold used during splitting
        #[arg(short, long)]
        threshold: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Split { secret, threshold, shares: n } => {
            let shares = split(secret.as_bytes(), threshold, n)
                .context("failed to split secret")?;

            println!("Split into {} shares (threshold: {})\n", n, threshold);
            for s in &shares {
                println!("{}", encode_share(s));
            }
        }

        Commands::Reconstruct { shares: strings, threshold } => {
            let shares: Vec<_> = strings.iter()
                .map(|s| decode_share(s))
                .collect::<std::result::Result<Vec<_>, _>>()
                .context("failed to parse shares")?;

            let secret = reconstruct(&shares, threshold)
                .context("failed to reconstruct")?;

            let text = String::from_utf8(secret)
                .context("reconstructed data is not valid UTF-8")?;

            println!("Reconstructed: {}", text);
        }
    }

    Ok(())
}
