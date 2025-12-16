//! ACP client implementation using SACP conductor
//!
//! This module provides the core ACP client functionality, building on the
//! SACP conductor for composable proxy chains.

use crate::acp::config::{AcpConfig, AgentConfig};
use crate::acp::tui::{AcpTui, AgentEvent};
use anyhow::Result;
use crossterm::event::{self, Event};
use sacp::schema::{
    ContentBlock, EnvVariable, InitializeRequest, NewSessionRequest, PromptRequest,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
    SessionNotification, SessionUpdate, TextContent, ToolCallStatus, VERSION as PROTOCOL_VERSION,
};
use sacp::{Component, DynComponent, JrConnectionCx};
use sacp_conductor::{Conductor, McpBridgeMode};
use sacp_tokio::AcpAgent;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

/// Options for running the ACP client
pub struct AcpClientOptions {
    /// Agent to connect to (by name from config)
    pub agent_name: Option<String>,
    /// Command override (takes precedence over agent_name)
    pub command_override: Option<String>,
    /// Single prompt to run (non-interactive mode)
    pub prompt: Option<String>,
    /// Run in agent mode (deciduous becomes the agent for an editor)
    pub agent_mode: bool,
    /// Enable trace logging to a directory
    pub trace_dir: Option<PathBuf>,
    /// Log level for stderr output
    pub log_level: Option<tracing::Level>,
    /// Disable TUI (use simple stdin/stdout)
    pub no_tui: bool,
}

impl Default for AcpClientOptions {
    fn default() -> Self {
        Self {
            agent_name: None,
            command_override: None,
            prompt: None,
            agent_mode: false,
            trace_dir: None,
            log_level: None,
            no_tui: false,
        }
    }
}

/// Run the ACP client with the specified options
///
/// This is the main entry point for `deciduous acp`.
pub async fn run_acp_client(options: AcpClientOptions) -> Result<()> {
    // Set up logging if requested
    if let Some(level) = options.log_level {
        use tracing_subscriber::EnvFilter;
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new(level.to_string()))
            .with_writer(io::stderr)
            .init();
    }

    if options.agent_mode {
        run_agent_mode(options).await
    } else {
        run_client_mode(options).await
    }
}

/// Run in client mode - connect to an agent and interact
async fn run_client_mode(options: AcpClientOptions) -> Result<()> {
    // Resolve agent configuration
    let agent_config = resolve_agent_config(
        options.agent_name.as_deref(),
        options.command_override.as_deref(),
    )?;

    eprintln!(
        "Connecting to agent: {} {}",
        agent_config.command,
        agent_config.args.join(" ")
    );

    // Create the AcpAgent from the config
    let agent = create_acp_agent(&agent_config)?;

    tracing::debug!("Agent server: {:?}", agent.server());

    // If single prompt mode, run non-interactively
    if let Some(prompt) = options.prompt {
        run_single_prompt_simple(agent, &prompt).await
    } else if options.no_tui {
        // Simple stdin/stdout mode
        run_interactive_simple(agent).await
    } else {
        // Full TUI mode
        run_interactive_tui(agent, &agent_config).await
    }
}

/// Run in agent mode - deciduous becomes the agent (for editors)
async fn run_agent_mode(options: AcpClientOptions) -> Result<()> {
    // In agent mode, we wrap a downstream agent with deciduous capabilities
    let agent_config = resolve_agent_config(
        options.agent_name.as_deref(),
        options.command_override.as_deref(),
    )?;

    eprintln!(
        "Starting deciduous agent wrapping: {} {}",
        agent_config.command,
        agent_config.args.join(" ")
    );

    let _agent = create_acp_agent(&agent_config)?;

    let deciduous = DeciduousComponent::new();

    let mut conductor = Conductor::new(
        "deciduous-agent".to_string(),
        move |init_req| {
            let deciduous = deciduous.clone();
            async move {
                tracing::info!("Building deciduous agent chain");

                let components = vec![DynComponent::new(deciduous)];

                Ok((init_req, components))
            }
        },
        McpBridgeMode::default(),
    );

    // Enable tracing if requested
    if let Some(trace_dir) = options.trace_dir {
        std::fs::create_dir_all(&trace_dir)?;
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let trace_path = trace_dir.join(format!("{}.jsons", timestamp));
        conductor = conductor
            .trace_to_path(&trace_path)
            .map_err(|e| anyhow::anyhow!("Failed to set up tracing: {}", e))?;
        tracing::info!("Tracing to {}", trace_path.display());
    }

    // Serve on stdio (editor connects to us)
    conductor
        .run(sacp_tokio::Stdio::new())
        .await
        .map_err(|e| anyhow::anyhow!("Conductor error: {}", e))
}

/// The deciduous component - injects decision tracking capabilities
#[derive(Clone)]
struct DeciduousComponent {
    // Future: Add deciduous database connection, MCP tool registry, etc.
}

impl DeciduousComponent {
    fn new() -> Self {
        Self {}
    }
}

impl Component for DeciduousComponent {
    async fn serve(self, client: impl Component) -> Result<(), sacp::Error> {
        // For MVP: just pass through to the client
        // Future: intercept messages, inject tools, log conversations
        tracing::debug!("DeciduousComponent::serve starting");

        // For now, just forward everything
        // This is where we'll add:
        // - MCP tool injection for deciduous_add_*, deciduous_link, etc.
        // - Conversation logging
        // - Context preservation
        client.serve(sacp_tokio::Stdio::new()).await
    }
}

/// Create an AcpAgent from agent config
fn create_acp_agent(config: &AgentConfig) -> Result<AcpAgent> {
    // Build the McpServer::Stdio configuration
    let server = sacp::schema::McpServer::Stdio {
        name: config.name.clone().unwrap_or_else(|| config.command.clone()),
        command: PathBuf::from(&config.command),
        args: config.args.clone(),
        env: config
            .env
            .iter()
            .map(|(k, v)| EnvVariable {
                name: k.clone(),
                value: v.clone(),
                meta: None,
            })
            .collect(),
    };

    Ok(AcpAgent::new(server))
}

/// Simpler interactive mode using direct ClientToAgent
async fn run_interactive_simple(agent: AcpAgent) -> Result<()> {
    use sacp::role::ClientToAgent;

    let (stdin, stdout, _stderr, mut child) = agent
        .spawn_process()
        .map_err(|e| anyhow::anyhow!("Failed to spawn agent process: {}", e))?;

    let transport = sacp::ByteStreams::new(stdin.compat_write(), stdout.compat());

    let result = ClientToAgent::builder()
        .name("deciduous-acp")
        .on_receive_notification(handle_session_notification)
        .on_receive_request(handle_permission_request)
        .with_client(transport, |cx| run_interactive_session(cx))
        .await;

    let _ = child.kill().await;
    result.map_err(|e| anyhow::anyhow!("ACP client error: {}", e))
}

/// TUI-based interactive mode
async fn run_interactive_tui(agent: AcpAgent, config: &AgentConfig) -> Result<()> {
    use crate::acp::tui::{restore_terminal, setup_terminal};
    use sacp::role::ClientToAgent;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Setup terminal
    let mut terminal = setup_terminal()
        .map_err(|e| anyhow::anyhow!("Failed to setup terminal: {}", e))?;

    // Create TUI state
    let mut tui = AcpTui::new();

    // Create channels for communication
    let (event_tx, event_rx) = mpsc::channel::<AgentEvent>();
    let (prompt_tx, prompt_rx) = mpsc::channel::<String>();
    tui.set_event_receiver(event_rx);

    // Spawn the agent process
    let (agent_stdin, agent_stdout, _stderr, mut child) = agent
        .spawn_process()
        .map_err(|e| anyhow::anyhow!("Failed to spawn agent process: {}", e))?;

    let transport = sacp::ByteStreams::new(agent_stdin.compat_write(), agent_stdout.compat());

    // Wrap prompt_rx for async access
    let prompt_rx = Arc::new(Mutex::new(prompt_rx));
    let agent_name = config.name.clone().unwrap_or_else(|| config.command.clone());
    let event_tx_clone = event_tx.clone();

    // Run ACP client in background task
    let acp_handle = tokio::spawn(async move {
        let prompt_rx = prompt_rx;
        let event_tx = event_tx_clone;
        let agent_name = agent_name;

        // Create notification handler that sends to our channel
        let event_tx_notif = event_tx.clone();

        let result = ClientToAgent::builder()
            .name("deciduous-acp-tui")
            .on_receive_notification(move |notification: SessionNotification, _cx| {
                let event_tx = event_tx_notif.clone();
                async move {
                    handle_tui_notification(notification, &event_tx);
                    Ok(())
                }
            })
            .on_receive_request(handle_permission_request)
            .with_client(transport, |cx: JrConnectionCx<ClientToAgent>| {
                let prompt_rx = prompt_rx.clone();
                let event_tx = event_tx.clone();
                let agent_name = agent_name.clone();
                async move {
                    run_tui_session(cx, prompt_rx, event_tx, agent_name).await
                }
            })
            .await;

        result
    });

    // Main TUI event loop
    let result = run_tui_loop(&mut terminal, &mut tui, &prompt_tx).await;

    // Cleanup
    let _ = child.kill().await;
    acp_handle.abort();

    restore_terminal(&mut terminal)
        .map_err(|e| anyhow::anyhow!("Failed to restore terminal: {}", e))?;

    result
}

/// Run the TUI event loop
async fn run_tui_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    tui: &mut AcpTui,
    prompt_tx: &mpsc::Sender<String>,
) -> Result<()> {
    loop {
        // Process any pending agent events
        tui.process_agent_events();

        // Draw the UI
        terminal.draw(|f| tui.render(f))?;

        // Check for user input with timeout
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if let Some(prompt) = tui.on_key(key) {
                        // User submitted a prompt
                        let _ = prompt_tx.send(prompt);
                    }
                }
                Event::Mouse(mouse) => {
                    tui.on_mouse(mouse);
                }
                Event::Resize(_, _) => {
                    // Terminal resized, will redraw on next iteration
                }
                _ => {}
            }
        }

        if tui.should_quit() {
            break;
        }
    }

    Ok(())
}

/// Handle notifications and send events to TUI
fn handle_tui_notification(notification: SessionNotification, event_tx: &mpsc::Sender<AgentEvent>) {
    match &notification.update {
        SessionUpdate::AgentMessageChunk(chunk) => {
            if let Some(text) = extract_text(&chunk.content) {
                let _ = event_tx.send(AgentEvent::TextChunk(text));
            }
        }
        SessionUpdate::AgentThoughtChunk(chunk) => {
            if let Some(text) = extract_text(&chunk.content) {
                let _ = event_tx.send(AgentEvent::ThoughtChunk(text));
            }
        }
        SessionUpdate::ToolCall(tool_call) => {
            let _ = event_tx.send(AgentEvent::ToolCallStart {
                id: tool_call.id.to_string(),
                title: tool_call.title.clone(),
            });
        }
        SessionUpdate::ToolCallUpdate(update) => {
            if let Some(status) = &update.fields.status {
                let status_str = match status {
                    ToolCallStatus::Pending => "pending",
                    ToolCallStatus::InProgress => "in_progress",
                    ToolCallStatus::Completed => "completed",
                    ToolCallStatus::Failed => "failed",
                };

                if *status == ToolCallStatus::Completed {
                    let result = update.fields.content.as_ref()
                        .and_then(|c| c.first())
                        .map(|item| match item {
                            sacp::schema::ToolCallContent::Content { content } => {
                                extract_text(content).unwrap_or_default()
                            }
                            _ => String::new(),
                        })
                        .unwrap_or_default();

                    let _ = event_tx.send(AgentEvent::ToolCallComplete {
                        id: update.id.to_string(),
                        result,
                    });
                } else {
                    let _ = event_tx.send(AgentEvent::ToolCallUpdate {
                        id: update.id.to_string(),
                        status: status_str.to_string(),
                    });
                }
            }
        }
        _ => {}
    }
}

/// Extract text from a ContentBlock
fn extract_text(block: &ContentBlock) -> Option<String> {
    match block {
        ContentBlock::Text(text) => Some(text.text.clone()),
        _ => None,
    }
}

/// Run the TUI session - handles initialization and prompt loop
async fn run_tui_session(
    cx: JrConnectionCx<sacp::role::ClientToAgent>,
    prompt_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<String>>>,
    event_tx: mpsc::Sender<AgentEvent>,
    agent_name: String,
) -> Result<(), sacp::Error> {
    // Send initializing event
    let _ = event_tx.send(AgentEvent::Initializing);

    // Initialize the agent
    let init_response = cx
        .send_request(InitializeRequest {
            protocol_version: PROTOCOL_VERSION,
            client_capabilities: Default::default(),
            client_info: Default::default(),
            meta: None,
        })
        .block_task()
        .await?;

    let name = init_response
        .agent_info
        .as_ref()
        .map(|i| i.name.clone())
        .unwrap_or(agent_name);

    let _ = event_tx.send(AgentEvent::Initialized(name));

    // Create session
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let session_response = cx
        .send_request(NewSessionRequest {
            mcp_servers: vec![],
            cwd,
            meta: None,
        })
        .block_task()
        .await?;

    let session_id = session_response.session_id.clone();
    let _ = event_tx.send(AgentEvent::SessionCreated(session_id.to_string()));

    // Prompt loop - wait for prompts from TUI
    loop {
        // Check for new prompts (non-blocking with small sleep)
        let prompt = {
            let rx = prompt_rx.lock().await;
            rx.try_recv().ok()
        };

        if let Some(prompt) = prompt {
            // Send the prompt to the agent
            let _response = cx
                .send_request(PromptRequest {
                    session_id: session_id.clone(),
                    prompt: vec![ContentBlock::Text(TextContent {
                        text: prompt,
                        annotations: None,
                        meta: None,
                    })],
                    meta: None,
                })
                .block_task()
                .await?;

            // Signal message complete
            let _ = event_tx.send(AgentEvent::MessageComplete);
        }

        // Small delay to prevent busy loop
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Simpler single-prompt mode
async fn run_single_prompt_simple(agent: AcpAgent, prompt: &str) -> Result<()> {
    use sacp::role::ClientToAgent;

    let (stdin, stdout, _stderr, mut child) = agent
        .spawn_process()
        .map_err(|e| anyhow::anyhow!("Failed to spawn agent process: {}", e))?;

    let transport = sacp::ByteStreams::new(stdin.compat_write(), stdout.compat());
    let prompt = prompt.to_string();

    let result = ClientToAgent::builder()
        .name("deciduous-acp")
        .on_receive_notification(handle_session_notification)
        .on_receive_request(handle_permission_request)
        .with_client(transport, |cx: JrConnectionCx<ClientToAgent>| {
            let prompt = prompt.clone();
            async move {
                // Initialize
                let _ = cx
                    .send_request(InitializeRequest {
                        protocol_version: PROTOCOL_VERSION,
                        client_capabilities: Default::default(),
                        client_info: Default::default(),
                        meta: None,
                    })
                    .block_task()
                    .await?;

                // Create session
                let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
                let session_response = cx
                    .send_request(NewSessionRequest {
                        mcp_servers: vec![],
                        cwd,
                        meta: None,
                    })
                    .block_task()
                    .await?;

                // Send prompt
                let _response = cx
                    .send_request(PromptRequest {
                        session_id: session_response.session_id,
                        prompt: vec![ContentBlock::Text(TextContent {
                            text: prompt,
                            annotations: None,
                            meta: None,
                        })],
                        meta: None,
                    })
                    .block_task()
                    .await?;

                Ok(())
            }
        })
        .await;

    let _ = child.kill().await;
    result.map_err(|e| anyhow::anyhow!("ACP client error: {}", e))
}

/// Resolve agent configuration from various sources
fn resolve_agent_config(
    agent_name: Option<&str>,
    command_override: Option<&str>,
) -> Result<AgentConfig> {
    // Command override takes highest precedence
    if let Some(cmd) = command_override {
        return AgentConfig::from_command_string(cmd).map_err(|e| anyhow::anyhow!("{}", e));
    }

    // Load config and merge with built-in defaults
    // This ensures built-in agents (opencode, claude-code, elizacp) are always available
    let defaults = AcpConfig::with_defaults();
    let user_config = AcpConfig::load();
    let config = defaults.merge(user_config);

    // If agent name specified, look it up
    if let Some(name) = agent_name {
        return config.get_agent(name).cloned().ok_or_else(|| {
            let available = config.list_agents().join(", ");
            anyhow::anyhow!(
                "Agent '{}' not found in config. Available: {}",
                name,
                if available.is_empty() {
                    "(none)"
                } else {
                    &available
                }
            )
        });
    }

    // Try default agent from config (user's default takes precedence if set)
    if let Some(agent) = config.get_default_agent() {
        return Ok(agent.clone());
    }

    Err(anyhow::anyhow!(
        "No agent configured. Use --agent <name> or --command, or set default_agent in config.\n\
         Available agents: {}",
        config.list_agents().join(", ")
    ))
}

/// Handle session notifications from the agent (streaming updates)
async fn handle_session_notification(
    notification: SessionNotification,
    _cx: JrConnectionCx<sacp::role::ClientToAgent>,
) -> Result<(), sacp::Error> {
    match &notification.update {
        SessionUpdate::AgentMessageChunk(chunk) => {
            // Print the streamed text content
            print_content_block(&chunk.content);
            let _ = io::stdout().flush();
        }
        SessionUpdate::AgentThoughtChunk(chunk) => {
            // Print agent's internal reasoning (to stderr)
            eprint_content_block(&chunk.content);
            let _ = io::stderr().flush();
        }
        SessionUpdate::UserMessageChunk(chunk) => {
            // Echo back user message chunks (usually not needed)
            print_content_block(&chunk.content);
            let _ = io::stdout().flush();
        }
        SessionUpdate::ToolCall(tool_call) => {
            eprintln!("\n[Tool Call: {}]", tool_call.title);
        }
        SessionUpdate::ToolCallUpdate(update) => {
            if let Some(status) = &update.fields.status {
                match status {
                    ToolCallStatus::Pending => {
                        // Tool is pending
                    }
                    ToolCallStatus::InProgress => {
                        // Tool is still running
                    }
                    ToolCallStatus::Completed => {
                        // Tool completed - check for content in fields
                        if let Some(content) = &update.fields.content {
                            for item in content {
                                match item {
                                    sacp::schema::ToolCallContent::Content { content } => {
                                        eprintln!("[Tool Result]");
                                        print_content_block(content);
                                    }
                                    sacp::schema::ToolCallContent::Diff { diff: _ } => {
                                        eprintln!("[Tool Result: <diff>]");
                                    }
                                    sacp::schema::ToolCallContent::Terminal { terminal_id: _ } => {
                                        eprintln!("[Tool Result: <terminal>]");
                                    }
                                }
                            }
                        }
                    }
                    ToolCallStatus::Failed => {
                        eprintln!("[Tool Failed]");
                    }
                }
            }
        }
        SessionUpdate::Plan(plan) => {
            eprintln!("\n[Plan: {} entries]", plan.entries.len());
            for entry in &plan.entries {
                eprintln!("  - {}", entry.content);
            }
        }
        SessionUpdate::AvailableCommandsUpdate(_) => {
            // Commands available changed - usually not interesting to display
            tracing::debug!("Available commands updated");
        }
        SessionUpdate::CurrentModeUpdate(mode) => {
            eprintln!("\n[Mode changed: {}]", mode.current_mode_id);
        }
    }
    Ok(())
}

/// Print a ContentBlock to stdout
fn print_content_block(block: &ContentBlock) {
    match block {
        ContentBlock::Text(text) => {
            print!("{}", text.text);
        }
        ContentBlock::Image(_) => {
            print!("[image]");
        }
        ContentBlock::Audio(_) => {
            print!("[audio]");
        }
        ContentBlock::ResourceLink(link) => {
            print!("[resource: {}]", link.uri);
        }
        ContentBlock::Resource(resource) => {
            print!("[embedded resource]");
            tracing::debug!("Resource: {:?}", resource);
        }
    }
}

/// Print a ContentBlock to stderr
fn eprint_content_block(block: &ContentBlock) {
    match block {
        ContentBlock::Text(text) => {
            eprint!("{}", text.text);
        }
        ContentBlock::Image(_) => {
            eprint!("[image]");
        }
        ContentBlock::Audio(_) => {
            eprint!("[audio]");
        }
        ContentBlock::ResourceLink(link) => {
            eprint!("[resource: {}]", link.uri);
        }
        ContentBlock::Resource(resource) => {
            eprint!("[embedded resource]");
            tracing::debug!("Resource: {:?}", resource);
        }
    }
}

/// Handle permission requests from the agent (auto-approve for MVP)
async fn handle_permission_request(
    request: RequestPermissionRequest,
    request_cx: sacp::JrRequestCx<RequestPermissionResponse>,
    _cx: JrConnectionCx<sacp::role::ClientToAgent>,
) -> Result<(), sacp::Error> {
    // Display the tool call that needs permission
    eprintln!(
        "\n[Permission request for tool call: {}]",
        request.tool_call.id
    );

    let option_id = request.options.first().map(|opt| opt.id.clone());

    match option_id {
        Some(id) => {
            eprintln!("[Auto-approving option: {}]", id);
            request_cx.respond(RequestPermissionResponse {
                outcome: RequestPermissionOutcome::Selected { option_id: id },
                meta: None,
            })
        }
        None => {
            eprintln!("[No options provided, cancelling]");
            request_cx.respond(RequestPermissionResponse {
                outcome: RequestPermissionOutcome::Cancelled,
                meta: None,
            })
        }
    }
}

/// Run the interactive session
async fn run_interactive_session(
    cx: JrConnectionCx<sacp::role::ClientToAgent>,
) -> Result<(), sacp::Error> {
    // Initialize the agent
    eprintln!("Initializing agent...");
    let init_response = cx
        .send_request(InitializeRequest {
            protocol_version: PROTOCOL_VERSION,
            client_capabilities: Default::default(),
            client_info: Default::default(),
            meta: None,
        })
        .block_task()
        .await?;

    let agent_name = init_response
        .agent_info
        .as_ref()
        .map(|i| i.name.as_str())
        .unwrap_or("(unknown)");

    eprintln!("Agent initialized: {}", agent_name);

    // Create a new session
    eprintln!("Creating session...");
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let session_response = cx
        .send_request(NewSessionRequest {
            mcp_servers: vec![],
            cwd,
            meta: None,
        })
        .block_task()
        .await?;

    let session_id = session_response.session_id;
    eprintln!("Session created: {}", session_id);
    eprintln!("---");
    eprintln!("Enter prompts (Ctrl+D or /quit to exit):");
    eprintln!();

    // Interactive prompt loop
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        let _ = stdout.flush();

        let mut input = String::new();
        match stdin.lock().read_line(&mut input) {
            Ok(0) => {
                eprintln!("\nGoodbye!");
                break;
            }
            Ok(_) => {
                let prompt = input.trim();
                if prompt.is_empty() {
                    continue;
                }

                if prompt == "/quit" || prompt == "/exit" {
                    eprintln!("Goodbye!");
                    break;
                }

                let _response = cx
                    .send_request(PromptRequest {
                        session_id: session_id.clone(),
                        prompt: vec![ContentBlock::Text(TextContent {
                            text: prompt.to_string(),
                            annotations: None,
                            meta: None,
                        })],
                        meta: None,
                    })
                    .block_task()
                    .await?;

                println!();
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    Ok(())
}
