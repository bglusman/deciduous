//! ACP TUI - Rich terminal interface for ACP agent interactions
//!
//! Uses tui-chat widgets for the chat interface, integrated with
//! our ACP client for streaming agent responses.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::sync::mpsc;
use tui_chat::{ChatArea, ChatMessage, InputArea};

/// Messages from the ACP client to the TUI
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Agent is initializing
    Initializing,
    /// Agent initialized with name
    Initialized(String),
    /// Session created
    SessionCreated(String),
    /// Streaming text chunk from agent
    TextChunk(String),
    /// Agent thinking/reasoning chunk
    ThoughtChunk(String),
    /// Tool call started
    ToolCallStart { id: String, title: String },
    /// Tool call update
    ToolCallUpdate { id: String, status: String },
    /// Tool call completed with result
    ToolCallComplete { id: String, result: String },
    /// Agent message complete
    MessageComplete,
    /// Error occurred
    Error(String),
    /// Connection closed
    Disconnected,
}

/// The ACP TUI application state
pub struct AcpTui {
    chat_area: ChatArea,
    input_area: InputArea,
    status_line: String,
    agent_name: String,
    session_id: Option<String>,
    current_response: String,
    current_tool_calls: Vec<ToolCallState>,
    should_quit: bool,
    chat_rect: Rect,
    event_rx: Option<mpsc::Receiver<AgentEvent>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ToolCallState {
    id: String,
    title: String,
    status: String,
}

impl AcpTui {
    pub fn new() -> Self {
        Self {
            chat_area: ChatArea::new(),
            input_area: InputArea::new(),
            status_line: "Connecting...".to_string(),
            agent_name: "Agent".to_string(),
            session_id: None,
            current_response: String::new(),
            current_tool_calls: Vec::new(),
            should_quit: false,
            chat_rect: Rect::default(),
            event_rx: None,
        }
    }

    /// Set the event receiver for agent events
    pub fn set_event_receiver(&mut self, rx: mpsc::Receiver<AgentEvent>) {
        self.event_rx = Some(rx);
    }

    /// Process any pending agent events
    pub fn process_agent_events(&mut self) {
        // Collect events first to avoid borrow issues
        let events: Vec<AgentEvent> = self.event_rx
            .as_ref()
            .map(|rx| {
                let mut events = Vec::new();
                while let Ok(event) = rx.try_recv() {
                    events.push(event);
                }
                events
            })
            .unwrap_or_default();

        for event in events {
            self.handle_agent_event(event);
        }
    }

    fn handle_agent_event(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::Initializing => {
                self.status_line = "Initializing agent...".to_string();
            }
            AgentEvent::Initialized(name) => {
                self.agent_name = name.clone();
                self.status_line = format!("Connected to {}", name);
            }
            AgentEvent::SessionCreated(id) => {
                self.session_id = Some(id.clone());
                self.status_line = format!("{} | Session: {}", self.agent_name, &id[..id.len().min(12)]);
            }
            AgentEvent::TextChunk(text) => {
                self.current_response.push_str(&text);
            }
            AgentEvent::ThoughtChunk(_text) => {
                // Could show in a separate panel, for now ignore
            }
            AgentEvent::ToolCallStart { id, title } => {
                self.current_tool_calls.push(ToolCallState {
                    id,
                    title: title.clone(),
                    status: "running".to_string(),
                });
                self.status_line = format!("Running: {}", title);
            }
            AgentEvent::ToolCallUpdate { id, status } => {
                if let Some(tc) = self.current_tool_calls.iter_mut().find(|t| t.id == id) {
                    tc.status = status;
                }
            }
            AgentEvent::ToolCallComplete { id, result } => {
                if let Some(tc) = self.current_tool_calls.iter_mut().find(|t| t.id == id) {
                    tc.status = "completed".to_string();
                }
                // Append tool result to response if meaningful
                if !result.is_empty() && result.len() < 500 {
                    if !self.current_response.is_empty() {
                        self.current_response.push_str("\n");
                    }
                    self.current_response.push_str(&format!("[Tool: {}]", result));
                }
                self.status_line = format!("{} | Ready", self.agent_name);
            }
            AgentEvent::MessageComplete => {
                // Finalize the current response as a message
                if !self.current_response.trim().is_empty() {
                    self.chat_area.add_message(ChatMessage {
                        sender: self.agent_name.clone(),
                        content: self.current_response.trim().to_string(),
                    });
                }
                self.current_response.clear();
                self.current_tool_calls.clear();
                self.status_line = format!("{} | Ready", self.agent_name);
            }
            AgentEvent::Error(msg) => {
                self.chat_area.add_message(ChatMessage {
                    sender: "Error".to_string(),
                    content: msg,
                });
                self.status_line = format!("{} | Error occurred", self.agent_name);
            }
            AgentEvent::Disconnected => {
                self.status_line = "Disconnected".to_string();
            }
        }
    }

    /// Handle a key event, returns the user's message if Enter is pressed
    pub fn on_key(&mut self, key: event::KeyEvent) -> Option<String> {
        if key.kind != event::KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::SHIFT)
                    || key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    self.input_area.newline();
                    None
                } else {
                    let input = self.input_area.submit();
                    if !input.trim().is_empty() {
                        // Add user message to chat
                        self.chat_area.add_message(ChatMessage {
                            sender: "You".to_string(),
                            content: input.clone(),
                        });
                        self.status_line = format!("{} | Thinking...", self.agent_name);
                        Some(input)
                    } else {
                        None
                    }
                }
            }
            KeyCode::PageUp => {
                self.chat_area.scroll_up(5);
                None
            }
            KeyCode::PageDown => {
                self.chat_area.scroll_down(5);
                None
            }
            KeyCode::Esc => {
                self.should_quit = true;
                None
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                None
            }
            KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input_area.newline();
                None
            }
            KeyCode::Char(c) => {
                self.input_area.insert_char(c);
                None
            }
            KeyCode::Backspace => {
                self.input_area.backspace();
                None
            }
            KeyCode::Left => {
                self.input_area.cursor_left();
                None
            }
            KeyCode::Right => {
                self.input_area.cursor_right();
                None
            }
            KeyCode::Up => {
                self.input_area.cursor_up();
                None
            }
            KeyCode::Down => {
                self.input_area.cursor_down();
                None
            }
            _ => None,
        }
    }

    /// Handle mouse events
    pub fn on_mouse(&mut self, mouse: event::MouseEvent) {
        use event::MouseEventKind;

        // Check if mouse is within chat area
        if mouse.column >= self.chat_rect.x
            && mouse.column < self.chat_rect.x + self.chat_rect.width
            && mouse.row >= self.chat_rect.y
            && mouse.row < self.chat_rect.y + self.chat_rect.height
        {
            match mouse.kind {
                MouseEventKind::ScrollUp => self.chat_area.scroll_up(3),
                MouseEventKind::ScrollDown => self.chat_area.scroll_down(3),
                _ => {}
            }
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Render the TUI
    pub fn render(&mut self, frame: &mut Frame) {
        let size = frame.area();

        // Calculate input height dynamically
        let input_height = self.input_area.calculate_display_lines(size.width);

        // Layout: status bar (1) + chat area (flex) + input area (dynamic)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),           // Status bar
                Constraint::Min(5),              // Chat area
                Constraint::Length(input_height), // Input area
            ])
            .split(size);

        // Render status bar
        self.render_status_bar(frame, chunks[0]);

        // Store chat rect for mouse handling
        self.chat_rect = chunks[1];

        // Render chat area
        self.chat_area.render(frame, chunks[1]);

        // Render input area
        self.input_area.render(frame, chunks[2]);

        // Show streaming response if in progress
        if !self.current_response.is_empty() {
            self.render_streaming_indicator(frame, chunks[1]);
        }
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let status = Paragraph::new(Line::from(vec![
            Span::styled(
                " deciduous acp ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(&self.status_line, Style::default().fg(Color::Gray)),
            Span::raw(" | "),
            Span::styled(
                "Esc/Ctrl+C: quit",
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw(" | "),
            Span::styled(
                "PgUp/PgDn: scroll",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        frame.render_widget(status, area);
    }

    fn render_streaming_indicator(&self, frame: &mut Frame, chat_area: Rect) {
        // Show a small indicator that we're receiving a response
        // This overlays on the bottom of the chat area
        if chat_area.height < 3 {
            return;
        }

        let indicator_area = Rect {
            x: chat_area.x,
            y: chat_area.y + chat_area.height - 2,
            width: chat_area.width,
            height: 1,
        };

        let dots = match (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            / 500)
            % 4
        {
            0 => ".",
            1 => "..",
            2 => "...",
            _ => "",
        };

        let preview = if self.current_response.len() > 50 {
            format!("{}...", &self.current_response[..50])
        } else {
            self.current_response.clone()
        };

        let indicator = Paragraph::new(format!(" {} typing{} {}", self.agent_name, dots, preview))
            .style(Style::default().fg(Color::DarkGray));

        frame.render_widget(indicator, indicator_area);
    }
}

/// Setup the terminal for TUI mode
pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restore the terminal to normal mode
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
