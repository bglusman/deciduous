use chrono::Local;
use clap::{Parser, Subcommand};
use colored::Colorize;
use deciduous::Database;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "deciduous")]
#[command(author, version, about = "Decision graph tooling for AI-assisted development")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize deciduous in current directory
    Init,

    /// Add a new node to the decision graph
    Add {
        /// Node type: goal, decision, option, action, outcome, observation
        node_type: String,

        /// Title of the node
        title: String,

        /// Optional description
        #[arg(short, long)]
        description: Option<String>,

        /// Confidence level (0-100)
        #[arg(short, long)]
        confidence: Option<u8>,

        /// Git commit hash to link this node to
        #[arg(long)]
        commit: Option<String>,
    },

    /// Add an edge between nodes
    Link {
        /// Source node ID
        from: i32,

        /// Target node ID
        to: i32,

        /// Rationale for this connection
        #[arg(short, long)]
        rationale: Option<String>,

        /// Edge type: leads_to, requires, chosen, rejected, blocks, enables
        #[arg(short = 't', long, default_value = "leads_to")]
        edge_type: String,
    },

    /// Update node status
    Status {
        /// Node ID
        id: i32,

        /// New status: pending, active, completed, rejected
        status: String,
    },

    /// List all nodes
    Nodes,

    /// List all edges
    Edges,

    /// Export full graph as JSON
    Graph,

    /// Start the graph viewer server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },

    /// Export graph to JSON file
    Sync {
        /// Output path (default: .deciduous/web/graph-data.json)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Create a database backup
    Backup {
        /// Output path (default: deciduous_backup_<timestamp>.db)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show recent command log
    Commands {
        /// Number of commands to show
        #[arg(short, long, default_value = "20")]
        limit: i64,
    },
}

fn main() {
    let args = Args::parse();

    // Handle init separately - it doesn't need an existing database
    if let Command::Init = args.command {
        if let Err(e) = deciduous::init::init_project() {
            eprintln!("{} {}", "Error:".red(), e);
            std::process::exit(1);
        }
        return;
    }

    let db = match Database::open() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("{} Failed to open database: {}", "Error:".red(), e);
            std::process::exit(1);
        }
    };

    match args.command {
        Command::Init => unreachable!(), // Handled above
        Command::Add { node_type, title, description, confidence, commit } => {
            match db.create_node(&node_type, &title, description.as_deref(), confidence, commit.as_deref()) {
                Ok(id) => {
                    let conf_str = confidence.map(|c| format!(" [confidence: {}%]", c)).unwrap_or_default();
                    let commit_str = commit.as_ref().map(|c| format!(" [commit: {}]", &c[..7.min(c.len())])).unwrap_or_default();
                    println!("{} node {} (type: {}, title: {}){}{}",
                        "Created".green(), id, node_type, title, conf_str, commit_str);
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                    std::process::exit(1);
                }
            }
        }

        Command::Link { from, to, rationale, edge_type } => {
            match db.create_edge(from, to, &edge_type, rationale.as_deref()) {
                Ok(id) => {
                    println!("{} edge {} ({} -> {} via {})", "Created".green(), id, from, to, edge_type);
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                    std::process::exit(1);
                }
            }
        }

        Command::Status { id, status } => {
            match db.update_node_status(id, &status) {
                Ok(()) => println!("{} node {} status to '{}'", "Updated".green(), id, status),
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                    std::process::exit(1);
                }
            }
        }

        Command::Nodes => {
            match db.get_all_nodes() {
                Ok(nodes) => {
                    if nodes.is_empty() {
                        println!("No nodes found. Add one with: deciduous add goal \"My goal\"");
                    } else {
                        println!("{:<5} {:<12} {:<10} {}", "ID", "TYPE", "STATUS", "TITLE");
                        println!("{}", "-".repeat(70));
                        for n in nodes {
                            let type_colored = match n.node_type.as_str() {
                                "goal" => n.node_type.yellow(),
                                "decision" => n.node_type.cyan(),
                                "action" => n.node_type.green(),
                                "outcome" => n.node_type.blue(),
                                "observation" => n.node_type.magenta(),
                                _ => n.node_type.white(),
                            };
                            println!("{:<5} {:<12} {:<10} {}", n.id, type_colored, n.status, n.title);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                    std::process::exit(1);
                }
            }
        }

        Command::Edges => {
            match db.get_all_edges() {
                Ok(edges) => {
                    if edges.is_empty() {
                        println!("No edges found. Link nodes with: deciduous link 1 2 -r \"reason\"");
                    } else {
                        println!("{:<5} {:<6} {:<6} {:<12} {}", "ID", "FROM", "TO", "TYPE", "RATIONALE");
                        println!("{}", "-".repeat(70));
                        for e in edges {
                            println!(
                                "{:<5} {:<6} {:<6} {:<12} {}",
                                e.id,
                                e.from_node_id,
                                e.to_node_id,
                                e.edge_type,
                                e.rationale.unwrap_or_default()
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                    std::process::exit(1);
                }
            }
        }

        Command::Graph => {
            match db.get_graph() {
                Ok(graph) => {
                    match serde_json::to_string_pretty(&graph) {
                        Ok(json) => println!("{}", json),
                        Err(e) => {
                            eprintln!("{} Serializing graph: {}", "Error:".red(), e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                    std::process::exit(1);
                }
            }
        }

        Command::Serve { port } => {
            println!("{} Starting graph viewer at http://localhost:{}", "Deciduous".cyan(), port);
            if let Err(e) = deciduous::serve::start_graph_server(port) {
                eprintln!("{} Server error: {}", "Error:".red(), e);
                std::process::exit(1);
            }
        }

        Command::Sync { output } => {
            let output_path = output.unwrap_or_else(|| {
                PathBuf::from(".deciduous/web/graph-data.json")
            });

            // Create parent directories if needed
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            match db.get_graph() {
                Ok(graph) => {
                    match serde_json::to_string_pretty(&graph) {
                        Ok(json) => {
                            match std::fs::write(&output_path, json) {
                                Ok(()) => {
                                    println!("{} graph to {}", "Exported".green(), output_path.display());
                                    println!("  {} nodes, {} edges", graph.nodes.len(), graph.edges.len());
                                }
                                Err(e) => {
                                    eprintln!("{} Writing file: {}", "Error:".red(), e);
                                    std::process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{} Serializing graph: {}", "Error:".red(), e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                    std::process::exit(1);
                }
            }
        }

        Command::Backup { output } => {
            let db_path = Database::db_path();
            if !db_path.exists() {
                eprintln!("{} No database found at {}", "Error:".red(), db_path.display());
                std::process::exit(1);
            }

            let backup_path = output.unwrap_or_else(|| {
                let timestamp = Local::now().format("%Y%m%d_%H%M%S");
                PathBuf::from(format!("deciduous_backup_{}.db", timestamp))
            });

            match std::fs::copy(&db_path, &backup_path) {
                Ok(bytes) => {
                    println!("{} backup: {} ({} bytes)", "Created".green(), backup_path.display(), bytes);
                }
                Err(e) => {
                    eprintln!("{} Creating backup: {}", "Error:".red(), e);
                    std::process::exit(1);
                }
            }
        }

        Command::Commands { limit } => {
            match db.get_recent_commands(limit) {
                Ok(commands) => {
                    if commands.is_empty() {
                        println!("No commands logged.");
                    } else {
                        for c in commands {
                            println!(
                                "[{}] {} (exit: {})",
                                c.started_at,
                                truncate(&c.command, 60),
                                c.exit_code.map(|c| c.to_string()).unwrap_or_else(|| "running".to_string())
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
