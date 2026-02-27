use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use firelocal_core::FireLocal;
use owo_colors::OwoColorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[derive(Parser)]
#[command(name = "firelocal")]
#[command(version = "0.1.0")]
#[command(about = "FireLocal CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new FireLocal project
    Init {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Start an interactive shell
    Shell {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Push a document
    Put {
        key: String,
        value: String,
        #[arg(short, long, default_value = ".")]
        db_path: String,
    },
    /// Get a document
    Get {
        key: String,
        #[arg(short, long, default_value = ".")]
        db_path: String,
    },
    /// Flush memtable to SST
    Flush {
        #[arg(short, long, default_value = ".")]
        db_path: String,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Run compaction to merge SST files and remove tombstones
    Compact {
        #[arg(short, long, default_value = ".")]
        db_path: String,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Initialize/update .env configuration
    Init {
        #[arg(default_value = ".")]
        path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            println!("Initializing FireLocal in {}", path);
            std::fs::create_dir_all(&path)?;
            // Maybe create a default config?
            println!("{}", "Success!".green());
        }
        Commands::Put {
            key,
            value,
            db_path,
        } => {
            let mut db = FireLocal::new(db_path).context("Failed to open DB")?;
            // Load default rules for CLI usage
            db.load_rules("service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }")?;

            db.put(key.clone(), value.clone().into_bytes())
                .context("Failed to put")?;
            println!("Written {} = {}", key.green(), value);
        }
        Commands::Get { key, db_path } => {
            let mut db = FireLocal::new(db_path).context("Failed to open DB")?; // mut required for method signature if internal mutable? No, get is &self usually, but load_rules needs mut.
            db.load_rules("service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }")?;

            if let Ok(Some(bytes)) = db.get(&key) {
                if let Ok(s) = std::str::from_utf8(&bytes) {
                    println!("{}", s);
                } else {
                    println!("{:?}", bytes);
                }
            } else {
                println!("{}", "Not found".red());
            }
        }
        Commands::Flush { db_path } => {
            let mut db = FireLocal::new(db_path).context("Failed to open DB")?;
            db.load_rules("service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }")?;
            db.flush().context("Failed to flush Memtable")?;
            println!("{}", "Flushed Memtable to SST".green());
        }
        Commands::Shell { path } => {
            let mut rl = DefaultEditor::new()?;
            let history_file = ".firelocal_history";
            if rl.load_history(history_file).is_err() {
                // No history
            }

            println!("FireLocal Shell. Type 'exit' to quit.");

            let mut db = FireLocal::new(&path).context("Failed to open DB")?;
            db.load_rules("service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }")?;

            loop {
                let readline = rl.readline("firelocal> ");
                match readline {
                    Ok(line) => {
                        let _ = rl.add_history_entry(line.as_str());
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.is_empty() {
                            continue;
                        }

                        match parts[0] {
                            "exit" | "quit" => break,
                            "put" => {
                                if parts.len() < 3 {
                                    println!("Usage: put <key> <json_value>");
                                    continue;
                                }
                                let key = parts[1];
                                // Value might contain spaces, join remainder
                                let value = parts[2..].join(" ");
                                match db.put(key.to_string(), value.into_bytes()) {
                                    Ok(_) => println!("{}", "OK".green()),
                                    Err(e) => println!("Error: {}", e.red()),
                                }
                            }
                            "get" => {
                                if parts.len() < 2 {
                                    println!("Usage: get <key>");
                                    continue;
                                }
                                let key = parts[1];
                                if let Ok(Some(bytes)) = db.get(key) {
                                    let s = String::from_utf8_lossy(&bytes);
                                    println!("{}", s);
                                } else {
                                    println!("{}", "Not Found".red());
                                }
                            }
                            "del" => {
                                if parts.len() < 2 {
                                    println!("Usage: del <key>");
                                    continue;
                                }
                                let key = parts[1];
                                match db.delete(key.to_string()) {
                                    Ok(_) => println!("{}", "OK".green()),
                                    Err(e) => println!("Error: {}", e.red()),
                                }
                            }
                            "flush" => match db.flush() {
                                Ok(_) => println!("{}", "Flushed memtable to SST".green()),
                                Err(e) => println!("Error: {}", e.red()),
                            },
                            "compact" => match db.compact() {
                                Ok(stats) => {
                                    println!("{}", "Compaction completed!".green());
                                    println!(
                                        "  Files: {} → {}",
                                        stats.files_before, stats.files_after
                                    );
                                    println!("  Tombstones removed: {}", stats.tombstones_removed);
                                    println!(
                                        "  Size reduction: {:.2}%",
                                        stats.size_reduction_percent()
                                    );
                                }
                                Err(e) => println!("Error: {}", e.red()),
                            },
                            "help" => {
                                println!("Available commands:");
                                println!("  put <key> <json>  - Write document");
                                println!("  get <key>         - Read document");
                                println!("  del <key>         - Delete document");
                                println!("  flush             - Flush memtable to SST");
                                println!("  compact           - Run compaction");
                                println!("  help              - Show this help");
                                println!("  exit/quit         - Exit shell");
                            }
                            _ => println!("Unknown command. Type 'help' for available commands."),
                        }
                    }
                    Err(ReadlineError::Interrupted) => {
                        println!("CTRL-C");
                        break;
                    }
                    Err(ReadlineError::Eof) => {
                        println!("CTRL-D");
                        break;
                    }
                    Err(err) => {
                        println!("Error: {:?}", err);
                        break;
                    }
                }
            }
            rl.save_history(history_file)?;
        }
        Commands::Config { action } => match action {
            ConfigAction::Show { path } => {
                use firelocal_core::config::FireLocalConfig;
                match FireLocalConfig::load_or_create(Some(std::path::Path::new(&path))) {
                    Ok(config) => {
                        println!("{}", "Configuration:".green());
                        println!("  Project ID: {}", config.project_id);
                        println!("  DB Path: {}", config.db_path.display());
                        println!("  Sync Mode: {:?}", config.sync_mode);
                        println!("  Sync Interval: {}s", config.sync_interval);
                        if config.is_sync_enabled() {
                            println!(
                                "  Firebase Project: {}",
                                config.firebase_project_id.as_deref().unwrap_or("Not set")
                            );
                        }
                    }
                    Err(e) => println!("{}: {}", "Error".red(), e),
                }
            }
            ConfigAction::Init { path } => {
                use firelocal_core::config::FireLocalConfig;
                match FireLocalConfig::load_or_create(Some(std::path::Path::new(&path))) {
                    Ok(_) => println!(
                        "{}",
                        "Configuration initialized/updated successfully!".green()
                    ),
                    Err(e) => println!("{}: {}", "Error".red(), e),
                }
            }
        },
        Commands::Compact { db_path } => {
            let db = FireLocal::new(db_path).context("Failed to open DB")?;
            match db.compact() {
                Ok(stats) => {
                    println!("{}", "Compaction completed!".green());
                    println!("  Files: {} → {}", stats.files_before, stats.files_after);
                    println!(
                        "  Entries: {} → {}",
                        stats.entries_before, stats.entries_after
                    );
                    println!("  Tombstones removed: {}", stats.tombstones_removed);
                    println!("  Size: {} → {} bytes", stats.size_before, stats.size_after);
                    println!("  Reduction: {:.2}%", stats.size_reduction_percent());
                }
                Err(e) => println!("{}: {}", "Error".red(), e),
            }
        }
    }

    Ok(())
}
