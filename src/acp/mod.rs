//! ACP (Agent Client Protocol) support for deciduous
//!
//! This module provides a full SACP client that can connect to any ACP-compliant
//! agent (Claude Code, OpenCode, etc.) and inject deciduous tools for decision tracking.
//!
//! # Architecture
//!
//! Following the symposium pattern, deciduous acts as a composable component:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        User Interface                           │
//! │            (TUI, CLI, or future IDE integration)                │
//! └───────────────────────────────┬─────────────────────────────────┘
//!                                 │ ACP JSON-RPC (stdio)
//!                                 ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     SACP Conductor                              │
//! │              (orchestrates component chain)                     │
//! └───────────────────────────────┬─────────────────────────────────┘
//!                                 │
//!                                 ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                  Deciduous Component                            │
//! │  ┌─────────────────────────────────────────────────────────────┐│
//! │  │                    Tool Injector                            ││
//! │  │  - Injects deciduous_* tools via MCP-over-ACP               ││
//! │  │  - deciduous_add_*, deciduous_link, deciduous_query_*       ││
//! │  └─────────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────────┐│
//! │  │                 Conversation Logger                         ││
//! │  │  - Logs prompts/responses to decision graph                 ││
//! │  │  - Preserves context across sessions                        ││
//! │  └─────────────────────────────────────────────────────────────┘│
//! └───────────────────────────────┬─────────────────────────────────┘
//!                                 │
//!                                 ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     Base ACP Agent                              │
//! │              (claude-code, opencode, elizacp)                   │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage Modes
//!
//! ## 1. Interactive Client (TUI)
//! ```bash
//! deciduous acp                    # Use default agent from config
//! deciduous acp --agent opencode   # Use specific agent
//! ```
//!
//! ## 2. One-shot Prompt
//! ```bash
//! deciduous acp --prompt "Fix the bug in main.rs"
//! ```
//!
//! ## 3. Agent Mode (be the agent for an editor)
//! ```bash
//! deciduous acp --agent-mode -- claude --acp
//! ```
//!
//! # Configuration
//!
//! Config file: `~/.config/deciduous/config.toml` or `.deciduous/config.toml`
//!
//! ```toml
//! [acp]
//! default_agent = "opencode"
//!
//! [acp.agents.opencode]
//! command = "opencode"
//! args = ["agent", "--stdio"]
//!
//! [acp.agents.claude-code]
//! command = "claude"
//! args = ["--acp"]
//! ```

pub mod client;
pub mod config;
pub mod tui;

pub use client::run_acp_client;
pub use config::{AcpConfig, AgentConfig};
pub use tui::{AcpTui, AgentEvent};
