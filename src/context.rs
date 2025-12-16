//! Context management for multiple decision graphs per project
//!
//! Allows managing multiple decision graph databases within a single project,
//! each representing a different decision context (e.g., feature work, refactoring).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Error types for context operations
#[derive(Debug)]
pub enum ContextError {
    /// Context already exists
    AlreadyExists(String),
    /// Context not found
    NotFound(String),
    /// Cannot delete default context
    CannotDeleteDefault,
    /// IO error
    Io(std::io::Error),
    /// JSON parsing error
    Json(serde_json::Error),
    /// Invalid context name
    InvalidName(String),
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextError::AlreadyExists(name) => write!(f, "Context '{}' already exists", name),
            ContextError::NotFound(name) => write!(f, "Context '{}' not found", name),
            ContextError::CannotDeleteDefault => {
                write!(f, "Cannot delete the default context (deciduous.db)")
            }
            ContextError::Io(e) => write!(f, "IO error: {}", e),
            ContextError::Json(e) => write!(f, "JSON error: {}", e),
            ContextError::InvalidName(name) => write!(
                f,
                "Invalid context name '{}'. Use lowercase letters, numbers, and hyphens only.",
                name
            ),
        }
    }
}

impl std::error::Error for ContextError {}

impl From<std::io::Error> for ContextError {
    fn from(e: std::io::Error) -> Self {
        ContextError::Io(e)
    }
}

impl From<serde_json::Error> for ContextError {
    fn from(e: serde_json::Error) -> Self {
        ContextError::Json(e)
    }
}

/// Information about a single context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInfo {
    /// Relative path to the database file (e.g., "deciduous.db" or "contexts/auth.db")
    pub path: String,
    /// Whether this is the default context
    pub is_default: bool,
    /// Number of nodes (if known)
    pub node_count: Option<usize>,
    /// Last modified time as ISO string
    pub last_modified: Option<String>,
}

/// Session state for a context (stored in active.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSession {
    pub active_session_id: Option<i32>,
    pub last_accessed: String,
    pub last_agent: Option<String>,
    pub root_goal_id: Option<i32>,
}

/// Active context state file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveState {
    pub version: u32,
    pub current_context: String,
    pub contexts: HashMap<String, ContextSession>,
}

impl Default for ActiveState {
    fn default() -> Self {
        Self {
            version: 1,
            current_context: "deciduous.db".to_string(),
            contexts: HashMap::new(),
        }
    }
}

/// Context manager for a deciduous project
pub struct ContextManager {
    /// Path to the .deciduous directory
    deciduous_dir: PathBuf,
}

impl ContextManager {
    /// Create a new context manager for the given .deciduous directory
    pub fn new(deciduous_dir: PathBuf) -> Self {
        Self { deciduous_dir }
    }

    /// Find the .deciduous directory by walking up from current directory
    pub fn find() -> Option<Self> {
        let current_dir = std::env::current_dir().ok()?;
        let mut dir = current_dir.as_path();

        loop {
            let deciduous_dir = dir.join(".deciduous");
            if deciduous_dir.exists() && deciduous_dir.is_dir() {
                return Some(Self::new(deciduous_dir));
            }
            dir = dir.parent()?;
        }
    }

    /// Get path to the contexts directory
    fn contexts_dir(&self) -> PathBuf {
        self.deciduous_dir.join("contexts")
    }

    /// Get path to the active state file
    fn active_state_path(&self) -> PathBuf {
        self.deciduous_dir.join("active.json")
    }

    /// Validate a context name
    fn validate_name(name: &str) -> Result<(), ContextError> {
        if name.is_empty() {
            return Err(ContextError::InvalidName(name.to_string()));
        }

        // Allow "default" as a special case
        if name == "default" {
            return Ok(());
        }

        // Check for valid characters: lowercase letters, numbers, hyphens
        let is_valid = name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

        if !is_valid || name.starts_with('-') || name.ends_with('-') {
            return Err(ContextError::InvalidName(name.to_string()));
        }

        Ok(())
    }

    /// Get the database path for a context name
    pub fn context_db_path(&self, name: &str) -> PathBuf {
        if name == "default" || name == "deciduous.db" {
            self.deciduous_dir.join("deciduous.db")
        } else {
            self.contexts_dir().join(format!("{}.db", name))
        }
    }

    /// Get the relative path string for a context
    fn context_relative_path(&self, name: &str) -> String {
        if name == "default" || name == "deciduous.db" {
            "deciduous.db".to_string()
        } else {
            format!("contexts/{}.db", name)
        }
    }

    /// Load the active state file
    pub fn load_active_state(&self) -> Result<ActiveState, ContextError> {
        let path = self.active_state_path();
        if !path.exists() {
            return Ok(ActiveState::default());
        }

        let content = fs::read_to_string(&path)?;
        let state: ActiveState = serde_json::from_str(&content)?;
        Ok(state)
    }

    /// Save the active state file
    pub fn save_active_state(&self, state: &ActiveState) -> Result<(), ContextError> {
        let path = self.active_state_path();
        let content = serde_json::to_string_pretty(state)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// List all available contexts
    pub fn list_contexts(&self) -> Result<Vec<ContextInfo>, ContextError> {
        let mut contexts = Vec::new();

        // Always include the default context
        let default_path = self.deciduous_dir.join("deciduous.db");
        if default_path.exists() {
            contexts.push(ContextInfo {
                path: "deciduous.db".to_string(),
                is_default: true,
                node_count: None,
                last_modified: file_modified_time(&default_path),
            });
        }

        // List contexts in the contexts/ directory
        let contexts_dir = self.contexts_dir();
        if contexts_dir.exists() {
            for entry in fs::read_dir(&contexts_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("db") {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");

                    contexts.push(ContextInfo {
                        path: format!("contexts/{}.db", name),
                        is_default: false,
                        node_count: None,
                        last_modified: file_modified_time(&path),
                    });
                }
            }
        }

        Ok(contexts)
    }

    /// Get the current active context
    pub fn current_context(&self) -> Result<String, ContextError> {
        let state = self.load_active_state()?;
        Ok(state.current_context)
    }

    /// Create a new context
    pub fn create_context(&self, name: &str) -> Result<PathBuf, ContextError> {
        Self::validate_name(name)?;

        if name == "default" {
            return Err(ContextError::InvalidName(
                "Cannot create a context named 'default'".to_string(),
            ));
        }

        let db_path = self.context_db_path(name);

        if db_path.exists() {
            return Err(ContextError::AlreadyExists(name.to_string()));
        }

        // Ensure contexts directory exists
        let contexts_dir = self.contexts_dir();
        if !contexts_dir.exists() {
            fs::create_dir_all(&contexts_dir)?;
        }

        // The database will be created when first opened
        // For now, just return the path
        Ok(db_path)
    }

    /// Switch to a different context
    pub fn switch_context(&self, name: &str) -> Result<PathBuf, ContextError> {
        let normalized_name = if name == "default" {
            "deciduous.db"
        } else {
            name
        };

        let db_path = self.context_db_path(name);

        // Check if the context exists (for non-default contexts)
        if normalized_name != "deciduous.db" && !db_path.exists() {
            return Err(ContextError::NotFound(name.to_string()));
        }

        // Update active state
        let mut state = self.load_active_state()?;
        state.current_context = self.context_relative_path(name);

        // Update last_accessed for the context
        let now = chrono::Utc::now().to_rfc3339();
        state
            .contexts
            .entry(state.current_context.clone())
            .or_insert_with(|| ContextSession {
                active_session_id: None,
                last_accessed: now.clone(),
                last_agent: None,
                root_goal_id: None,
            })
            .last_accessed = now.clone();

        self.save_active_state(&state)?;

        Ok(db_path)
    }

    /// Delete a context
    pub fn delete_context(&self, name: &str) -> Result<(), ContextError> {
        Self::validate_name(name)?;

        if name == "default" || name == "deciduous.db" {
            return Err(ContextError::CannotDeleteDefault);
        }

        let db_path = self.context_db_path(name);

        if !db_path.exists() {
            return Err(ContextError::NotFound(name.to_string()));
        }

        // Remove the database file
        fs::remove_file(&db_path)?;

        // Update active state if this was the current context
        let mut state = self.load_active_state()?;
        let relative_path = self.context_relative_path(name);

        if state.current_context == relative_path {
            state.current_context = "deciduous.db".to_string();
        }

        // Remove from contexts map
        state.contexts.remove(&relative_path);

        self.save_active_state(&state)?;

        Ok(())
    }

    /// Get the path to the .deciduous directory
    pub fn deciduous_dir(&self) -> &Path {
        &self.deciduous_dir
    }
}

/// Get the last modified time of a file as an ISO string
fn file_modified_time(path: &Path) -> Option<String> {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|t| {
            let datetime: chrono::DateTime<chrono::Utc> = t.into();
            datetime.to_rfc3339()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_context() -> (TempDir, ContextManager) {
        let temp_dir = TempDir::new().unwrap();
        let deciduous_dir = temp_dir.path().join(".deciduous");
        fs::create_dir_all(&deciduous_dir).unwrap();

        // Create a default database file
        fs::write(deciduous_dir.join("deciduous.db"), "").unwrap();

        let manager = ContextManager::new(deciduous_dir);
        (temp_dir, manager)
    }

    #[test]
    fn test_validate_name() {
        assert!(ContextManager::validate_name("auth-system").is_ok());
        assert!(ContextManager::validate_name("ui-refactor").is_ok());
        assert!(ContextManager::validate_name("feature123").is_ok());
        assert!(ContextManager::validate_name("default").is_ok());

        assert!(ContextManager::validate_name("").is_err());
        assert!(ContextManager::validate_name("Auth-System").is_err()); // uppercase
        assert!(ContextManager::validate_name("-invalid").is_err()); // starts with hyphen
        assert!(ContextManager::validate_name("invalid-").is_err()); // ends with hyphen
        assert!(ContextManager::validate_name("has spaces").is_err());
    }

    #[test]
    fn test_list_contexts() {
        let (_temp, manager) = setup_test_context();

        let contexts = manager.list_contexts().unwrap();
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].path, "deciduous.db");
        assert!(contexts[0].is_default);
    }

    #[test]
    fn test_create_context() {
        let (_temp, manager) = setup_test_context();

        let path = manager.create_context("auth-system").unwrap();
        assert!(path.to_string_lossy().contains("contexts/auth-system.db"));

        // Touch the file to simulate database creation
        fs::write(&path, "").unwrap();

        // Creating again should fail
        assert!(matches!(
            manager.create_context("auth-system"),
            Err(ContextError::AlreadyExists(_))
        ));
    }

    #[test]
    fn test_switch_context() {
        let (_temp, manager) = setup_test_context();

        // Create and switch to a new context
        let db_path = manager.create_context("test-context").unwrap();
        // Touch the file so it exists
        fs::write(&db_path, "").unwrap();

        manager.switch_context("test-context").unwrap();

        let current = manager.current_context().unwrap();
        assert_eq!(current, "contexts/test-context.db");

        // Switch back to default
        manager.switch_context("default").unwrap();
        let current = manager.current_context().unwrap();
        assert_eq!(current, "deciduous.db");
    }

    #[test]
    fn test_delete_context() {
        let (_temp, manager) = setup_test_context();

        // Create and then delete
        let db_path = manager.create_context("to-delete").unwrap();
        fs::write(&db_path, "").unwrap();

        manager.delete_context("to-delete").unwrap();
        assert!(!db_path.exists());

        // Cannot delete default
        assert!(matches!(
            manager.delete_context("default"),
            Err(ContextError::CannotDeleteDefault)
        ));
    }

    #[test]
    fn test_active_state_persistence() {
        let (_temp, manager) = setup_test_context();

        // Create a context and switch to it
        let db_path = manager.create_context("persistent").unwrap();
        fs::write(&db_path, "").unwrap();
        manager.switch_context("persistent").unwrap();

        // Reload and check
        let state = manager.load_active_state().unwrap();
        assert_eq!(state.current_context, "contexts/persistent.db");
    }
}
