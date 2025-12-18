use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "orbit")]
#[command(about="Orbit — UX-first workspace awareness + safe exploration (project-local)", long_about=None)]
pub struct OrbitCli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, default_value = ".")]
    pub root: String,

    /// Output in JSON format (for scripting)
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Tui,
    Census {
        #[arg(long, default_value = "4")]
        depth: usize,
        #[arg(long)]
        since: Option<String>,
    },
    Status,
    Focus {
        #[arg(long)]
        add: Option<String>,
        #[arg(long)]
        remove: Option<String>,
        #[arg(long)]
        list: bool,
    },
    Snap {
        #[arg(short, long)]
        label: Option<String>,
    },
    Export,
}

pub fn run() -> Result<()> {
    let cli = OrbitCli::parse();
    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => crate::tui::run(&cli.root),
        Commands::Census { depth, since } => {
            crate::scan::census::run_census(&cli.root, depth, since.as_deref(), cli.json)
        }
        Commands::Status => crate::index::status::print_status(&cli.root, cli.json),
        Commands::Focus { add, remove, list } => {
            crate::index::focus::handle_focus(&cli.root, add, remove, list, cli.json)
        }
        Commands::Snap { label } => {
            crate::snapshot::quick::snapshot_pinned(&cli.root, label.as_deref())
        }
        Commands::Export => crate::export::all::export_all(&cli.root),
    }
}
