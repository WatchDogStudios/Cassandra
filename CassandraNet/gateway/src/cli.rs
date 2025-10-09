use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "cngateway", version, about = "CassandraNet Gateway Service")]
pub struct CliArgs {
    #[arg(long)]
    pub print_config: bool,
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Generate an HS256 JWT with provided subject (sub) claim (expires in 1h)
    GenToken {
        #[arg(long)]
        sub: String,
    },
    /// Print build & version metadata
    Version {
        #[arg(long)]
        json: bool,
    },
}
