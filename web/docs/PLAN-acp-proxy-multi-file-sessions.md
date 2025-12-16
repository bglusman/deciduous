# ACP Proxy & Multi-File Session Management - Design Document

## Overview

This document details the design for:
1. **SACP-compatible proxy** for deciduous to enforce decision tracking at the protocol level
2. **Multi-file session management** for managing multiple decision contexts per project
3. **Compaction discouragement** strategy to preserve full conversation context
4. **Cross-file linking** for referencing nodes across different deciduous databases
5. **Config hierarchy** supporting global and local configuration

---

## Problem Statement

### Current Limitations

1. **Agent prompting is brittle**: AGENTS.md relies on voluntary compliance; agents can ignore it
2. **Single database per project**: `.deciduous/deciduous.db` forces all decisions into one file
3. **Context loss on compaction**: When agents compact conversation history, earlier decisions are lost
4. **No session continuity**: Each agent invocation starts fresh with no awareness of previous sessions
5. **No cross-project references**: Can't link a decision in project A to one in project B
6. **Local config only**: No way to share global preferences across projects

### Goals

- **G1**: Move from prompt-level hints to protocol-level enforcement
- **G2**: Support multiple decision contexts per project (e.g., `auth.db`, `ui-refactor.db`)
- **G3**: Preserve full conversation history even when agents would normally compact
- **G4**: Enable session continuity - resume previous work seamlessly
- **G5**: Allow cross-file references between decision databases
- **G6**: Implement XDG-compliant config hierarchy with inheritance

---

## Part 1: Multi-File Session Management

### 1.1 File Naming & Organization

```
.deciduous/
├── config.toml              # Project-level config (existing)
├── deciduous.db             # Default/main decision graph (existing)
├── contexts/                # NEW: Named decision contexts
│   ├── auth-system.db       # Feature-specific graph
│   ├── ui-refactor.db       # Another context
│   └── legacy-cleanup.db    # Long-running initiative
├── sessions/                # NEW: Session state tracking
│   └── active.json          # Tracks active session per context
└── patches/                 # Existing: Export patches for sync
```

**Naming Convention:**
- Context names: lowercase, hyphen-separated (e.g., `auth-system`, `ui-refactor`)
- Auto-generated contexts use branch name: `contexts/feature-dark-mode.db`
- Session names: ISO timestamp + optional description

### 1.2 Context Management Commands

```bash
# List all contexts
deciduous context list
# Output:
# * deciduous.db (default, 147 nodes, active session)
#   contexts/auth-system.db (23 nodes)
#   contexts/ui-refactor.db (89 nodes, 2 days stale)

# Create new context
deciduous context create auth-system
deciduous context create auth-system --from-branch  # Auto-name from git branch

# Switch active context
deciduous context switch auth-system
deciduous context switch --default  # Back to deciduous.db

# Delete context (with confirmation)
deciduous context delete auth-system

# Archive context (move to archive/, exclude from listings)
deciduous context archive auth-system

# Import nodes from one context to another
deciduous context import auth-system --nodes 1-50 --into deciduous.db
```

### 1.3 Session Continuity

The schema already has `decision_sessions` and `session_nodes` tables. We'll extend this:

**New Schema Additions:**

```sql
-- Extend decision_sessions
ALTER TABLE decision_sessions ADD COLUMN context_file TEXT DEFAULT 'deciduous.db';
ALTER TABLE decision_sessions ADD COLUMN agent_id TEXT;  -- Which agent created this
ALTER TABLE decision_sessions ADD COLUMN prompt_hash TEXT;  -- Hash of triggering prompt

-- New table: track active sessions per context
CREATE TABLE active_context (
    context_file TEXT PRIMARY KEY,
    active_session_id INTEGER REFERENCES decision_sessions(id),
    last_accessed TEXT NOT NULL,
    last_agent TEXT
);
```

**Session State File (`.deciduous/sessions/active.json`):**

```json
{
  "version": 1,
  "default_context": "deciduous.db",
  "contexts": {
    "deciduous.db": {
      "active_session_id": 42,
      "last_accessed": "2025-01-15T10:30:00Z",
      "last_agent": "claude-code",
      "root_goal_id": 156
    },
    "contexts/auth-system.db": {
      "active_session_id": 3,
      "last_accessed": "2025-01-14T15:20:00Z",
      "last_agent": "opencode",
      "root_goal_id": 1
    }
  }
}
```

### 1.4 Default Behaviors

| Scenario | Behavior |
|----------|----------|
| `deciduous add goal "X"` (no active session) | Create new session, set as active |
| `deciduous add goal "X"` (active session exists) | Add to active session |
| New ACP proxy connection | Continue active session (always) |
| Context switch | Suspend current session, resume target's active session |
| `deciduous sync` | Export only active context (unless `--all`) |
| Session reaches warn_node_count | Inject reminder about `deciduous session close` |

### 1.5 Session Commands

```bash
# View current session
deciduous session
# Output: Session #42 in deciduous.db
#         Started: 2025-01-15 10:30 (2h ago)
#         Root goal: "Add dark mode toggle" (node 156)
#         Nodes in session: 23
#         Last activity: "Implemented theme context" (5m ago)

# List all sessions for current context
deciduous session list
deciduous session list --all-contexts

# Start a new session (closes current)
deciduous session new "Refactoring auth flow"

# Resume a previous session
deciduous session resume 41

# Close current session
deciduous session close --summary "Completed dark mode implementation"

# View session history
deciduous session history --context auth-system.db
```

---

## Part 2: Config Hierarchy

### 2.1 Config Locations (XDG-compliant)

```
1. ~/.config/deciduous/config.toml     # User global defaults
2. .deciduous/config.toml              # Project overrides (existing)
3. Environment variables               # Runtime overrides
4. CLI flags                           # Highest precedence
```

**Resolution Order:** CLI > ENV > Project > User Global > Defaults

### 2.2 Extended Config Schema

```toml
# ~/.config/deciduous/config.toml (global)

[defaults]
# Default confidence for new nodes
confidence = 70
# Auto-commit deciduous files
auto_commit = true
# Default editor for descriptions
editor = "nvim"

[branch]
main_branches = ["main", "master", "develop"]
auto_detect = true

[session]
# Warn when session gets large (no auto-close)
warn_node_count = 100
warn_tree_depth = 15
warn_interval_nodes = 50
# Prompt before creating new session
confirm_new_session = true

[acp]
# Default proxy behavior
auto_inject_tools = true
log_all_prompts = true
compaction_strategy = "preserve"  # "preserve" | "summarize" | "allow"

[github]
# Global default (can override per-project)
# commit_repo = "owner/repo"

[sync]
# Default export path
export_path = ".deciduous/web/graph-data.json"
# Include archived contexts in sync
include_archived = false

[tui]
# Theme: "dark" | "light" | "auto"
theme = "auto"
# Keybinding preset: "vim" | "emacs" | "default"
keybindings = "vim"
```

```toml
# .deciduous/config.toml (project-level override)

[github]
commit_repo = "myorg/myproject"

[session]
warn_node_count = 100
warn_tree_depth = 15

[branch]
main_branches = ["main", "develop", "staging"]  # Add staging for this project
```

### 2.3 Config Merge Strategy

**Section-level deep merge with local precedence:**

```rust
fn merge_configs(global: Config, local: Config) -> Config {
    Config {
        defaults: local.defaults.merge_with(global.defaults),
        branch: BranchConfig {
            main_branches: if local.branch.main_branches.is_empty() {
                global.branch.main_branches
            } else {
                local.branch.main_branches  // Replace, don't append
            },
            auto_detect: local.branch.auto_detect.or(global.branch.auto_detect),
        },
        session: local.session.merge_with(global.session),
        acp: local.acp.merge_with(global.acp),
        github: local.github.merge_with(global.github),
        // ... etc
    }
}
```

**Rules:**
- Scalar values: Local overrides global
- Arrays: Local replaces global entirely (no merge)
- Nested objects: Recursive merge
- `None`/unset: Falls through to global/default

---

## Part 3: Cross-File Linking

### 3.1 Reference Format

**URI Scheme:**
```
deciduous://[context-file]#[change_id]
deciduous://auth-system.db#chg_a1b2c3d4
deciduous://../other-project/.deciduous/deciduous.db#chg_x9y8z7
deciduous://~/projects/shared-decisions.db#chg_common123
```

**Short Form (within same project):**
```
@auth-system:chg_a1b2c3d4
@:chg_a1b2c3d4  # Current context, just the change_id
```

### 3.2 External Reference Table

```sql
CREATE TABLE external_refs (
    id INTEGER PRIMARY KEY,
    local_node_id INTEGER NOT NULL REFERENCES decision_nodes(id),
    ref_type TEXT NOT NULL,  -- "links_to" | "derived_from" | "supersedes" | "related"
    target_uri TEXT NOT NULL,  -- Full deciduous:// URI
    target_change_id TEXT NOT NULL,  -- Extracted for quick lookup
    rationale TEXT,
    created_at TEXT NOT NULL,
    -- Cached target info (may be stale)
    cached_title TEXT,
    cached_type TEXT,
    cache_updated_at TEXT
);

CREATE INDEX idx_external_refs_target ON external_refs(target_change_id);
```

### 3.3 Cross-File Commands

```bash
# Link local node to external node
deciduous link 42 --external deciduous://auth-system.db#chg_abc123 -r "Depends on auth decision"

# Link to node in different project
deciduous link 42 --external "deciduous://../shared/.deciduous/deciduous.db#chg_xyz" -r "Company-wide policy"

# View external references
deciduous refs 42
# Output:
# Node 42 references:
#   → @auth-system:chg_abc123 "Use JWT tokens" (decision)
#   ← @ui-refactor:chg_def456 "Theme context pattern" (links here)

# Resolve and cache external node info
deciduous refs resolve --all  # Refresh all external ref caches
```

### 3.4 Sync with External Refs

When running `deciduous sync`:
1. Export includes `external_refs` as a separate section
2. Change_ids remain stable across exports
3. Web viewer shows external refs with "external link" indicator
4. TUI shows external refs grayed out (can't navigate into them)

---

## Part 4: Compaction Discouragement Strategy

### 4.1 The Problem

AI agents periodically "compact" conversation history to stay within context limits:
- Early messages are summarized or dropped
- Decisions made early in a session become inaccessible
- Agent loses awareness of constraints/choices made earlier

### 4.2 Strategy: "Context Preservation Mode"

**Approach:** The proxy intercepts and modifies agent behavior rather than blocking compaction.

**Implementation Layers:**

#### Layer 1: System Message Injection

When proxy connects, inject a system message:

```
[DECIDUOUS CONTEXT PRESERVATION]
This session is tracked by deciduous. Full conversation history is preserved in
the decision graph at .deciduous/deciduous.db.

You do NOT need to maintain early context in your working memory - it is
preserved externally and can be queried at any time using deciduous tools.

When you would normally compact/summarize to free context space:
1. First, ensure key decisions are logged to deciduous (deciduous_add_*)
2. Then freely drop older context - deciduous preserves it
3. Use deciduous_query_context to recall earlier decisions when needed

Available context recovery tools:
- deciduous_query_context: Search previous decisions by keyword
- deciduous_session_summary: Get summary of current session's decisions
- deciduous_get_node: Retrieve full details of any logged decision
```

#### Layer 2: Compaction Boundary Detection

Monitor agent messages for compaction indicators:
- Phrases like "To summarize what we've done...", "Earlier in our conversation..."
- Increasing references to "previous decisions" without specifics
- Tool call patterns suggesting context search

When detected, the proxy:
1. Logs an `observation` node: "Agent approaching context boundary"
2. Injects a reminder about deciduous context tools
3. Optionally triggers `deciduous_session_summary` proactively

#### Layer 3: Full Prompt/Response Logging

**NEW TABLE:**

```sql
CREATE TABLE conversation_log (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL REFERENCES decision_sessions(id),
    turn_number INTEGER NOT NULL,
    role TEXT NOT NULL,  -- "user" | "assistant" | "system"
    content TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    tool_calls_json TEXT,  -- JSON array of tool calls if any
    timestamp TEXT NOT NULL,
    token_estimate INTEGER,  -- Rough token count for context tracking
    compacted_at TEXT,  -- NULL until this turn is "compacted"
    compaction_summary TEXT  -- If compacted, the summary that replaced it
);

CREATE INDEX idx_conversation_log_session ON conversation_log(session_id, turn_number);
```

**Logging Behavior:**
- Every prompt/response is logged immediately
- When agent compacts, mark old turns with `compacted_at` timestamp
- Store the compaction summary alongside
- Full history remains queryable

#### Layer 4: Context Recovery Tools

Injected via MCP-over-ACP:

```typescript
// deciduous_query_context
// Searches conversation_log and decision_nodes for matching content
{
  name: "deciduous_query_context",
  description: "Search your preserved conversation history and decisions",
  parameters: {
    query: string,       // Search term
    scope: "all" | "conversation" | "decisions",
    session: "current" | "all" | number,  // Session ID
    limit: number
  }
}

// deciduous_session_summary
// Returns structured summary of current session
{
  name: "deciduous_session_summary",
  description: "Get a summary of decisions made in the current session",
  parameters: {
    include_conversation: boolean,  // Include key conversation excerpts
    format: "brief" | "detailed"
  }
}

// deciduous_recall_turn
// Retrieve a specific conversation turn
{
  name: "deciduous_recall_turn",
  description: "Retrieve the full content of a previous conversation turn",
  parameters: {
    turn_number: number,
    session_id?: number
  }
}
```

### 4.3 Compaction Strategy Config

```toml
[acp.compaction]
# "preserve": Log everything, encourage agent to use deciduous for context
# "summarize": When compaction detected, auto-generate decision summary
# "allow": Don't interfere, just log what happens
strategy = "preserve"

# Inject context preservation system message
inject_system_message = true

# Proactively surface session summary when context is ~80% full
proactive_summary_threshold = 0.8

# Log full prompts/responses (may be large)
log_full_conversation = true

# Compress old conversation logs after N days
compress_after_days = 7
```

---

## Part 5: ACP Proxy Architecture

### 5.1 Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        ACP Client                                │
│            (Editor, deciduous-tui, other tooling)                │
└───────────────────────────────┬─────────────────────────────────┘
                                │ ACP JSON-RPC (stdio/HTTP)
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                     SACP Conductor                               │
│           sacp conductor deciduous-proxy base-agent              │
└───────────────────────────────┬─────────────────────────────────┘
                                │ _proxy/successor/* protocol
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Deciduous Proxy Component                      │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Message Interceptor                       ││
│  │  - Log all prompts/responses to conversation_log            ││
│  │  - Detect compaction boundaries                             ││
│  │  - Inject system messages                                   ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                     Tool Injector                            ││
│  │  - Inject deciduous_* tools via MCP-over-ACP                ││
│  │  - Handle tool calls against local deciduous.db             ││
│  │  - deciduous_add_*, deciduous_link, deciduous_query_*       ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                   Session Manager                            ││
│  │  - Continue/create sessions on connect                      ││
│  │  - Track active context                                     ││
│  │  - Handle context switching                                 ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                   Git Integration                            ││
│  │  - Detect commits, auto-link to actions/outcomes            ││
│  │  - Branch tracking for context association                  ││
│  └─────────────────────────────────────────────────────────────┘│
└───────────────────────────────┬─────────────────────────────────┘
                                │ _proxy/successor/* protocol
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Base ACP Agent                               │
│              (claude-code-acp, opencode, etc.)                   │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Injected Tools (MCP-over-ACP)

```typescript
// Core decision tracking
deciduous_add_goal(title, confidence?, prompt?)
deciduous_add_decision(title, confidence?)
deciduous_add_option(title, decision_id, confidence?)
deciduous_add_action(title, confidence?, commit?)
deciduous_add_outcome(title, confidence?, commit?)
deciduous_add_observation(title, confidence?)

// Linking
deciduous_link(from_id, to_id, rationale?, edge_type?)

// Querying
deciduous_nodes(filter?, branch?, limit?)
deciduous_edges(node_id?)
deciduous_get_node(id)
deciduous_query_context(query, scope?, session?)

// Session management
deciduous_session_summary(include_conversation?, format?)
deciduous_session_info()
deciduous_recall_turn(turn_number, session_id?)

// Context switching
deciduous_context_switch(context_name)
deciduous_context_list()
```

### 5.3 Proxy Startup Sequence

```
1. Load config (global → project → env → cli)
2. Determine active context from .deciduous/sessions/active.json
3. Connect to deciduous database (create if needed)
4. Resume or create session based on staleness threshold
5. Register as SACP successor for base agent
6. Inject MCP tools into capability handshake
7. Inject system message about deciduous context preservation
8. Begin message interception loop
```

---

## Part 6: Git Integration

### 6.1 Default Tracking Behavior

**Invert gitignore assumption:**

Current `.gitignore`:
```
.deciduous/deciduous.db
```

Proposed approach - track by default:
```toml
# .deciduous/config.toml
[git]
# Which files to track in git
track_database = true  # .deciduous/deciduous.db
track_contexts = true  # .deciduous/contexts/*.db
track_patches = true   # .deciduous/patches/*.json (already tracked)
track_sessions = false # .deciduous/sessions/ (local state)
```

**Migration:** Provide command `deciduous git setup` that:
1. Removes `deciduous.db` from `.gitignore`
2. Adds `.deciduous/sessions/` to `.gitignore`
3. Stages `deciduous.db` for next commit

### 6.2 Commit Association

When proxy detects a git commit (via filesystem watch or hook):

```bash
# Auto-log action with commit
deciduous add action "Committed: $COMMIT_MSG" --commit $COMMIT_HASH --auto

# If active session has pending actions, link them
for action in $(deciduous nodes --session current --type action --no-commit); do
    deciduous update $action --commit $COMMIT_HASH
done
```

---

## Implementation Phases

### Phase 1: Multi-File Foundation (P0)
- [ ] Extend schema with `context_file` column
- [ ] Add `contexts/` directory support
- [ ] Implement `deciduous context {list,create,switch,delete}`
- [ ] Add `active.json` session state tracking
- [ ] Update `deciduous add` to respect active context

### Phase 2: Config Hierarchy (P1)
- [ ] Add global config support (`~/.config/deciduous/`)
- [ ] Implement config merge logic
- [ ] Add `[session]` and `[acp]` config sections
- [ ] CLI flags for runtime overrides

### Phase 3: Session Continuity (P1)
- [ ] Extend `decision_sessions` schema
- [ ] Implement session staleness detection
- [ ] Add `deciduous session {new,resume,close,list}`
- [ ] Auto-continue behavior in CLI

### Phase 4: Conversation Logging (P1)
- [ ] Add `conversation_log` table
- [ ] Implement logging in proxy message handler
- [ ] Add compaction detection heuristics
- [ ] Implement `deciduous_query_context` tool

### Phase 5: ACP Proxy Core (P0)
- [ ] Create `deciduous-acp-proxy` crate
- [ ] Implement SACP successor protocol
- [ ] Message interception and logging
- [ ] Tool injection via MCP-over-ACP

### Phase 6: Cross-File Linking (P2)
- [ ] Add `external_refs` table
- [ ] Implement URI scheme parsing
- [ ] Add `deciduous link --external` command
- [ ] Display external refs in TUI/web

### Phase 7: Git Integration (P2)
- [ ] Filesystem watch for commits
- [ ] Auto-association logic
- [ ] `deciduous git setup` migration command
- [ ] Update `.gitignore` defaults

---

## Design Decisions (Resolved)

### 1. Database Locking: Simple Lock File

**Decision:** One agent/interface owns the database at a time. Use a simple lock file.

**Rationale:** Multiple simultaneous access patterns (TUI + CLI + proxy) aren't real usage - you use one interface at a time. No need for WAL complexity or daemon overhead.

**Implementation:**
```rust
fn acquire_lock() -> Result<LockGuard, Error> {
    let lock_path = get_deciduous_dir()?.join("deciduous.lock");
    let lock_file = File::create(&lock_path)?;

    match lock_file.try_lock_exclusive() {
        Ok(_) => {
            write!(&lock_file, "{}", std::process::id())?;
            Ok(LockGuard { file: lock_file, path: lock_path })
        }
        Err(_) => {
            let pid = std::fs::read_to_string(&lock_path).unwrap_or("unknown".into());
            Err(Error::DatabaseLocked { pid })
        }
    }
}
```

**Behavior:**
- Short-lived commands (`deciduous add`, `deciduous nodes`) acquire lock briefly
- Long-lived processes (TUI, proxy) hold lock for duration
- Clear error message if another process holds the lock

### 2. Sessions: No Auto-Close, Add Size Warnings

**Decision:** Sessions never auto-close. Instead, warn when graphs get large.

**Rationale:** Sessions represent logical work units, not time windows. The user/agent should decide when work is "done."

**Implementation:**
```toml
[session]
# No auto-close, but warn when graph gets large
warn_node_count = 100      # "Session has 100+ nodes, consider closing"
warn_tree_depth = 15       # "Decision tree is 15 levels deep"
warn_interval_nodes = 50   # Re-warn every N nodes after threshold
```

The proxy injects a gentle reminder when thresholds are crossed.

### 3. Cross-Project References: Organic Support Only

**Decision:** Support the URI scheme with file paths (relative or absolute), but no special tooling.

**Rationale:** Cross-project workflows are edge cases. If paths break when projects move, that's acceptable - we handle broken links gracefully anyway.

**Implementation:**
- Parse `deciduous://path/to/file.db#change_id` URIs
- Relative paths resolved from current .deciduous directory
- Broken links shown as "unresolved" in UI, not errors

### 4. Conversation Log Size: Defer to Roadmap

**Decision:** Log everything now, add size management later.

**Roadmap items:**
- [ ] Configurable max conversation log size
- [ ] Tree trimming / archival support
- [ ] Compression for old sessions

---

## Open Questions (Remaining)

1. **Compaction detection accuracy**: How reliably can we detect compaction?
   - False positives: Agent summarizing for user (not compacting)
   - False negatives: Agent silently drops context without explicit summary

---

## Success Criteria

1. **Session continuity**: Agent reconnects and immediately knows prior context
2. **No lost decisions**: Every decision made in conversation is captured
3. **Context recovery**: Agent can query 100-turn-old decisions accurately
4. **Multi-context switching**: Switch contexts in <1s, no data loss
5. **Cross-file navigation**: TUI shows external refs, can open linked contexts
6. **Git hygiene**: Decision graphs committed alongside code changes

---

## References

- [SACP Architecture](https://symposium.org/sacp/architecture)
- [ACP Protocol Spec](https://github.com/anthropics/acp)
- [MCP-over-ACP](https://symposium.org/sacp/mcp-bridging)
- [Deciduous Schema](../src/schema.rs)
- [jj Dual-ID Model](https://martinvonz.github.io/jj/latest/working-copy/)
