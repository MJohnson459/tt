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
    /// Manually add a past session
    Add {
        /// Client name (uses default if omitted)
        client: Option<String>,
        #[arg(short, long, help = "Start time: HH:MM (today) or \"YYYY-MM-DD HH:MM\"")]
        start: String,
        #[arg(short, long, help = "End time: HH:MM (today) or \"YYYY-MM-DD HH:MM\"")]
        end: String,
        #[arg(short, long, help = "Session note")]
        note: Option<String>,
    },
    /// Append a note to the active session
    Note {
        #[arg(trailing_var_arg = true, num_args = 1..)]
        text: Vec<String>,
    },
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
    /// Open the raw data file in $EDITOR
    Edit,
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
        #[arg(short, long, help = "Hourly rate, optionally prefixed with a currency symbol (e.g. £75, €90, $100, 80)")]
        rate: String,
        #[arg(short, long, help = "ISO currency code — overrides any symbol in --rate (e.g. GBP, EUR, USD)")]
        currency: Option<String>,
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
        Command::Add { client, start, end, note } => {
            let mut store = data::Store::load()?;
            cmd::add_session(&mut store, client, &start, &end, note)?;
            store.save()?;
        }
        Command::Note { text } => {
            let mut store = data::Store::load()?;
            cmd::note(&mut store, text.join(" "))?;
            store.save()?;
        }
        Command::Edit => cmd::edit()?,
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
                ClientCmd::Add { name, rate, currency } => cmd::client_add(&mut store, name, &rate, currency)?,
                ClientCmd::List => cmd::client_list(&store),
                ClientCmd::Remove { name } => cmd::client_remove(&mut store, name)?,
                ClientCmd::Default { name } => cmd::client_default(&mut store, name)?,
            }
            store.save()?;
        }
    }
    Ok(())
}
