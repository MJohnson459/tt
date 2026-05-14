mod cmd;
mod data;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tt", about = "Time tracker for contractors", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Clock in to start a session
    In {
        /// Client name (uses default if omitted)
        client: Option<String>,
        #[arg(short, long, help = "Session note")]
        note: Option<String>,
    },
    /// Clock out and end the current session
    Out {
        #[arg(short, long, help = "Session note")]
        note: Option<String>,
    },
    /// Show current session status
    Status,
    /// List sessions
    Log {
        #[arg(short, long, help = "Filter by client")]
        client: Option<String>,
        #[arg(short, long, help = "This week only")]
        week: bool,
        #[arg(short, long, help = "This month only")]
        month: bool,
    },
    /// Show earnings summary grouped by client
    Summary {
        #[arg(short, long, help = "This week")]
        week: bool,
        #[arg(short, long, help = "This month")]
        month: bool,
    },
    /// Manage clients and hourly rates
    Client {
        #[command(subcommand)]
        action: ClientCmd,
    },
}

#[derive(Subcommand)]
enum ClientCmd {
    /// Add or update a client
    Add {
        name: String,
        #[arg(short, long)]
        rate: f64,
    },
    /// List all clients
    List,
    /// Remove a client
    Remove { name: String },
    /// Set the default client (used when none is specified at clock-in)
    Default { name: String },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::In { client, note } => {
            let mut store = data::Store::load()?;
            cmd::clock_in(&mut store, client, note)?;
            store.save()?;
        }
        Command::Out { note } => {
            let mut store = data::Store::load()?;
            cmd::clock_out(&mut store, note)?;
            store.save()?;
        }
        Command::Status => {
            let store = data::Store::load()?;
            cmd::status(&store);
        }
        Command::Log {
            client,
            week,
            month,
        } => {
            let store = data::Store::load()?;
            cmd::log(&store, client, week, month);
        }
        Command::Summary { week, month } => {
            let store = data::Store::load()?;
            cmd::summary(&store, week, month);
        }
        Command::Client { action } => {
            let mut store = data::Store::load()?;
            match action {
                ClientCmd::Add { name, rate } => cmd::client_add(&mut store, name, rate)?,
                ClientCmd::List => cmd::client_list(&store),
                ClientCmd::Remove { name } => cmd::client_remove(&mut store, name)?,
                ClientCmd::Default { name } => cmd::client_default(&mut store, name)?,
            }
            store.save()?;
        }
    }
    Ok(())
}
