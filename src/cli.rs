use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::index::{session, whitelist};
use crate::index::session::OrbitSession;

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

    /// Dry-run (where supported) to preview actions without writing
    #[arg(long, global = true)]
    pub dry_run: bool,
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
    Whitelist {
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
    /// Headless scan + export for CI (writes shared index/session)
    Ci {
        #[arg(long, default_value = "4")]
        depth: usize,
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        no_export: bool,
    },
}

pub fn run() -> Result<()> {
    let cli = OrbitCli::parse();
    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => crate::tui::run(&cli.root, cli.dry_run),
        Commands::Census { depth, since } => crate::scan::census::run_census(
            &cli.root,
            depth,
            since.as_deref(),
            cli.json,
            None,
        )
        .map(|_| ()),
        Commands::Status => crate::index::status::print_status(&cli.root, cli.json),
        Commands::Focus { add, remove, list } => {
            crate::index::focus::handle_focus(&cli.root, add, remove, list, cli.json)
        }
        Commands::Whitelist { add, remove, list } => {
            whitelist::handle_whitelist(&cli.root, add, remove, list, cli.json)
        }
        Commands::Snap { label } => {
            crate::snapshot::quick::snapshot_pinned(&cli.root, label.as_deref(), cli.dry_run)
        }
        Commands::Export => crate::export::all::export_all(&cli.root, cli.dry_run),
        Commands::Ci { depth, since, no_export } => {
            crate::scan::census::run_census(
                &cli.root,
                depth,
                since.as_deref(),
                cli.json,
                None,
            )?;
            if !no_export {
                crate::export::all::export_all(&cli.root, cli.dry_run)?;
            }
            // Save shared session marker for downstream consumers
            let sess = OrbitSession {
                version: 1,
                root: cli.root.clone(),
                lens: Some("projects".into()),
                search: None,
                selection: None,
                high_contrast: None,
            };
            let _ = session::save_session(&sess);
            Ok(())
        }
    }
}
