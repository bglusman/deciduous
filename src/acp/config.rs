//! ACP client configuration
//!
//! Supports both global (~/.config/deciduous/config.toml) and local (.deciduous/config.toml)
//! configuration for agent settings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Top-level ACP configuration section
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AcpConfig {
    /// Default agent to use when none specified
    #[serde(default)]
    pub default_agent: Option<String>,

    /// Agent configurations by name
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
}

/// Configuration for a single ACP agent
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AgentConfig {
    /// Command to run (e.g., "claude", "opencode")
    pub command: String,

    /// Arguments to pass to the command
    #[serde(default)]
    pub args: Vec<String>,

    /// Environment variables to set
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Optional display name for the agent
    #[serde(default)]
    pub name: Option<String>,
}

impl AgentConfig {
    /// Create a new agent config from a command string
    ///
    /// Parses shell-style command strings like "opencode agent --stdio"
    pub fn from_command_string(cmd: &str) -> Result<Self, String> {
        let parts = shell_words::split(cmd)
            .map_err(|e| format!("Failed to parse command: {}", e))?;

        if parts.is_empty() {
            return Err("Command string cannot be empty".into());
        }

        Ok(Self {
            command: parts[0].clone(),
            args: parts[1..].to_vec(),
            env: HashMap::new(),
            name: None,
        })
    }
}

impl AcpConfig {
    /// Load ACP config, merging global and local configs
    ///
    /// Priority: local > global > defaults
    pub fn load() -> Self {
        let global = Self::load_global().unwrap_or_default();
        let local = Self::load_local().unwrap_or_default();
        global.merge(local)
    }

    /// Load global config from ~/.config/deciduous/config.toml
    fn load_global() -> Option<Self> {
        let config_dir = dirs::config_dir()?;
        let config_path = config_dir.join("deciduous").join("config.toml");
        Self::load_from_path(&config_path)
    }

    /// Load local config from .deciduous/config.toml
    fn load_local() -> Option<Self> {
        let deciduous_dir = find_deciduous_dir()?;
        let config_path = deciduous_dir.join("config.toml");
        Self::load_from_path(&config_path)
    }

    /// Load config from a specific path
    fn load_from_path(path: &PathBuf) -> Option<Self> {
        let contents = std::fs::read_to_string(path).ok()?;

        // Parse the full config file and extract the [acp] section
        #[derive(Deserialize)]
        struct FullConfig {
            #[serde(default)]
            acp: AcpConfig,
        }

        toml::from_str::<FullConfig>(&contents)
            .ok()
            .map(|c| c.acp)
    }

    /// Merge two configs, with `other` taking precedence
    pub fn merge(mut self, other: Self) -> Self {
        // Other's default_agent takes precedence if set
        if other.default_agent.is_some() {
            self.default_agent = other.default_agent;
        }

        // Merge agents, other takes precedence
        for (name, config) in other.agents {
            self.agents.insert(name, config);
        }

        self
    }

    /// Get the default agent config
    pub fn get_default_agent(&self) -> Option<&AgentConfig> {
        self.default_agent
            .as_ref()
            .and_then(|name| self.agents.get(name))
    }

    /// Get a specific agent config by name
    pub fn get_agent(&self, name: &str) -> Option<&AgentConfig> {
        self.agents.get(name)
    }

    /// List all available agent names
    pub fn list_agents(&self) -> Vec<&str> {
        self.agents.keys().map(|s| s.as_str()).collect()
    }

    /// Create a config with sensible defaults for common agents
    pub fn with_defaults() -> Self {
        let mut agents = HashMap::new();

        // Claude Code (hypothetical - adjust based on actual CLI)
        agents.insert(
            "claude-code".to_string(),
            AgentConfig {
                command: "claude".to_string(),
                args: vec!["--acp".to_string()],
                env: HashMap::new(),
                name: Some("Claude Code".to_string()),
            },
        );

        // OpenCode
        agents.insert(
            "opencode".to_string(),
            AgentConfig {
                command: "opencode".to_string(),
                args: vec!["acp".to_string()],
                env: HashMap::new(),
                name: Some("OpenCode".to_string()),
            },
        );

        // Elizacp (for testing)
        agents.insert(
            "elizacp".to_string(),
            AgentConfig {
                command: "elizacp".to_string(),
                args: vec![],
                env: HashMap::new(),
                name: Some("Eliza (test agent)".to_string()),
            },
        );

        Self {
            default_agent: Some("elizacp".to_string()),
            agents,
        }
    }
}

/// Find the .deciduous directory by walking up the directory tree
fn find_deciduous_dir() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    let mut dir = current_dir.as_path();

    loop {
        let deciduous_path = dir.join(".deciduous");
        if deciduous_path.is_dir() {
            return Some(deciduous_path);
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_from_command_string() {
        let config = AgentConfig::from_command_string("opencode agent --stdio").unwrap();
        assert_eq!(config.command, "opencode");
        assert_eq!(config.args, vec!["agent", "--stdio"]);
    }

    #[test]
    fn test_agent_config_empty_command() {
        let result = AgentConfig::from_command_string("");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_with_defaults() {
        let config = AcpConfig::with_defaults();
        assert!(config.agents.contains_key("opencode"));
        assert!(config.agents.contains_key("elizacp"));
        assert_eq!(config.default_agent, Some("elizacp".to_string()));
    }

    #[test]
    fn test_config_merge() {
        let base = AcpConfig {
            default_agent: Some("agent1".to_string()),
            agents: {
                let mut m = HashMap::new();
                m.insert("agent1".to_string(), AgentConfig {
                    command: "cmd1".to_string(),
                    args: vec![],
                    env: HashMap::new(),
                    name: None,
                });
                m
            },
        };

        let override_cfg = AcpConfig {
            default_agent: Some("agent2".to_string()),
            agents: {
                let mut m = HashMap::new();
                m.insert("agent2".to_string(), AgentConfig {
                    command: "cmd2".to_string(),
                    args: vec![],
                    env: HashMap::new(),
                    name: None,
                });
                m
            },
        };

        let merged = base.merge(override_cfg);
        assert_eq!(merged.default_agent, Some("agent2".to_string()));
        assert!(merged.agents.contains_key("agent1"));
        assert!(merged.agents.contains_key("agent2"));
    }
}
