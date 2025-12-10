//! Project initialization for deciduous
//!
//! `deciduous init` creates all the files needed for decision graph tracking

use colored::Colorize;
use std::fs;
use std::path::Path;

/// Templates embedded at compile time
const DECISION_MD: &str = r#"---
description: Manage decision graph - track algorithm choices and reasoning
allowed-tools: Bash(deciduous:*)
argument-hint: <action> [args...]
---

# Decision Graph Management

**Log decisions IN REAL-TIME as you work, not retroactively.**

## When to Use This

| You're doing this... | Log this type | Command |
|---------------------|---------------|---------|
| Starting a new feature | `goal` | `/decision add goal "Add user auth"` |
| Choosing between approaches | `decision` | `/decision add decision "Choose auth method"` |
| Considering an option | `option` | `/decision add option "JWT tokens"` |
| About to write code | `action` | `/decision add action "Implementing JWT"` |
| Noticing something | `observation` | `/decision add obs "Found existing auth code"` |
| Finished something | `outcome` | `/decision add outcome "JWT working"` |

## Quick Commands

Based on $ARGUMENTS:

### View Commands
- `nodes` or `list` -> `deciduous nodes`
- `edges` -> `deciduous edges`
- `graph` -> `deciduous graph`
- `commands` -> `deciduous commands`

### Create Nodes (with optional confidence)
- `add goal <title>` -> `deciduous add goal "<title>" -c 90`
- `add decision <title>` -> `deciduous add decision "<title>" -c 75`
- `add option <title>` -> `deciduous add option "<title>" -c 70`
- `add action <title>` -> `deciduous add action "<title>" -c 85`
- `add obs <title>` -> `deciduous add observation "<title>" -c 80`
- `add outcome <title>` -> `deciduous add outcome "<title>" -c 90`

### Create Edges
- `link <from> <to> [reason]` -> `deciduous link <from> <to> -r "<reason>"`

### Sync Graph
- `sync` -> `deciduous sync`

## Node Types

| Type | Purpose | Example |
|------|---------|---------|
| `goal` | High-level objective | "Add user authentication" |
| `decision` | Choice point with options | "Choose auth method" |
| `option` | Possible approach | "Use JWT tokens" |
| `action` | Something implemented | "Added JWT middleware" |
| `outcome` | Result of action | "JWT auth working" |
| `observation` | Finding or data point | "Existing code uses sessions" |

## Edge Types

| Type | Meaning |
|------|---------|
| `leads_to` | Natural progression |
| `chosen` | Selected option |
| `rejected` | Not selected (include reason!) |
| `requires` | Dependency |
| `blocks` | Preventing progress |
| `enables` | Makes something possible |

## The Rule

```
LOG BEFORE YOU CODE, NOT AFTER.
SYNC BEFORE YOU PUSH.
```
"#;

const CONTEXT_MD: &str = r#"---
description: Recover context from decision graph and recent activity - USE THIS ON SESSION START
allowed-tools: Bash(deciduous:*, git:*)
argument-hint: [focus-area]
---

# Context Recovery

**RUN THIS AT SESSION START.** The decision graph is your persistent memory.

## Step 1: Query the Graph

```bash
# See all decisions (look for recent ones and pending status)
deciduous nodes

# See how decisions connect
deciduous edges

# What commands were recently run?
deciduous commands
```

## Step 2: Check Git State

```bash
git status
git log --oneline -10
git diff --stat
```

## After Gathering Context, Report:

1. **Current branch** and pending changes
2. **Recent decisions** (especially pending/active ones)
3. **Last actions** from git log and command log
4. **Open questions** or unresolved observations
5. **Suggested next steps**

---

## REMEMBER: Real-Time Logging Required

After recovering context, you MUST follow the logging workflow:

```
EVERY USER REQUEST -> Log goal/decision first
BEFORE CODE CHANGES -> Log action
AFTER CHANGES -> Log outcome, link nodes
BEFORE GIT PUSH -> deciduous sync
```

### Quick Logging Commands

```bash
deciduous add goal "What we're trying to do" -c 90
deciduous add action "What I'm about to implement" -c 85
deciduous add outcome "What happened" -c 95
deciduous link FROM TO -r "Connection reason"
deciduous sync  # Do this frequently!
```

---

## The Memory Loop

```
SESSION START
    |
Run /context -> See past decisions
    |
DO WORK -> Log BEFORE each action
    |
AFTER CHANGES -> Log outcomes, observations
    |
BEFORE PUSH -> deciduous sync
    |
PUSH -> Graph persists
    |
SESSION END -> Graph survives
    |
(repeat)
```

---

## Why This Matters

- Context loss during compaction loses your reasoning
- The graph survives - query it early, query it often
- Retroactive logging misses details - log in the moment
"#;

const CLAUDE_MD_SECTION: &str = r#"
## Decision Graph Workflow

**THIS IS MANDATORY. Log decisions IN REAL-TIME, not retroactively.**

### The Core Rule

```
BEFORE you do something -> Log what you're ABOUT to do
AFTER it succeeds/fails -> Log the outcome
ALWAYS -> Sync frequently so the graph updates
```

### Behavioral Triggers - MUST LOG WHEN:

| Trigger | Log Type | Example |
|---------|----------|---------|
| User asks for a new feature | `goal` | "Add dark mode" |
| Choosing between approaches | `decision` | "Choose state management" |
| About to write/edit code | `action` | "Implementing Redux store" |
| Something worked or failed | `outcome` | "Redux integration successful" |
| Notice something interesting | `observation` | "Existing code uses hooks" |

### Quick Commands

```bash
deciduous add goal "Title" -c 90
deciduous add decision "Title" -c 75
deciduous add action "Title" -c 85
deciduous link FROM TO -r "reason"
deciduous serve   # View live
deciduous sync    # Export for static hosting
```

### Session Start Checklist

Every new session, run `/context` or:

```bash
deciduous nodes    # What decisions exist?
deciduous edges    # How are they connected?
git status         # Current state
git log -10        # Recent commits
```
"#;

/// Initialize deciduous in the current directory
pub fn init_project() -> Result<(), String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("Could not get current directory: {}", e))?;

    println!("\n{}", "Initializing Deciduous...".cyan().bold());
    println!("   Directory: {}\n", cwd.display());

    // 1. Create .deciduous directory
    let deciduous_dir = cwd.join(".deciduous");
    create_dir_if_missing(&deciduous_dir)?;

    // 2. Initialize database by opening it (creates tables)
    let db_path = deciduous_dir.join("deciduous.db");
    println!("   {} {}", "Creating".green(), ".deciduous/deciduous.db");

    // Touch the DB path - the Database::open() will create it
    // We need to set the env var so Database::open() uses this path
    std::env::set_var("DECIDUOUS_DB_PATH", &db_path);

    // 3. Create .claude/commands directory
    let claude_dir = cwd.join(".claude").join("commands");
    create_dir_if_missing(&claude_dir)?;

    // 4. Write decision.md
    let decision_path = claude_dir.join("decision.md");
    write_file_if_missing(&decision_path, DECISION_MD, ".claude/commands/decision.md")?;

    // 5. Write context.md
    let context_path = claude_dir.join("context.md");
    write_file_if_missing(&context_path, CONTEXT_MD, ".claude/commands/context.md")?;

    // 6. Append to or create CLAUDE.md
    let claude_md_path = cwd.join("CLAUDE.md");
    append_claude_md(&claude_md_path)?;

    // 7. Add .deciduous to .gitignore if not already there
    add_to_gitignore(&cwd)?;

    println!("\n{}", "Deciduous initialized!".green().bold());
    println!("\nNext steps:");
    println!("  1. Run {} to start the graph viewer", "deciduous serve".cyan());
    println!("  2. Use {} to recover context at session start", "/context".cyan());
    println!("  3. Use {} to log decisions as you work", "/decision".cyan());
    println!();

    Ok(())
}

fn create_dir_if_missing(path: &Path) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| format!("Could not create {}: {}", path.display(), e))?;
        println!("   {} {}", "Creating".green(), path.display());
    }
    Ok(())
}

fn write_file_if_missing(path: &Path, content: &str, display_name: &str) -> Result<(), String> {
    if path.exists() {
        println!("   {} {} (already exists)", "Skipping".yellow(), display_name);
    } else {
        fs::write(path, content)
            .map_err(|e| format!("Could not write {}: {}", display_name, e))?;
        println!("   {} {}", "Creating".green(), display_name);
    }
    Ok(())
}

fn append_claude_md(path: &Path) -> Result<(), String> {
    let marker = "## Decision Graph Workflow";

    if path.exists() {
        let existing = fs::read_to_string(path)
            .map_err(|e| format!("Could not read CLAUDE.md: {}", e))?;

        if existing.contains(marker) {
            println!("   {} CLAUDE.md (workflow section already present)", "Skipping".yellow());
            return Ok(());
        }

        // Append the section
        let new_content = format!("{}\n{}", existing.trim_end(), CLAUDE_MD_SECTION);
        fs::write(path, new_content)
            .map_err(|e| format!("Could not update CLAUDE.md: {}", e))?;
        println!("   {} CLAUDE.md (added workflow section)", "Updated".green());
    } else {
        // Create new CLAUDE.md
        let content = format!("# Project Instructions\n{}", CLAUDE_MD_SECTION);
        fs::write(path, content)
            .map_err(|e| format!("Could not create CLAUDE.md: {}", e))?;
        println!("   {} CLAUDE.md", "Creating".green());
    }

    Ok(())
}

fn add_to_gitignore(cwd: &Path) -> Result<(), String> {
    let gitignore_path = cwd.join(".gitignore");
    let entry = ".deciduous/";

    if gitignore_path.exists() {
        let existing = fs::read_to_string(&gitignore_path)
            .map_err(|e| format!("Could not read .gitignore: {}", e))?;

        if existing.lines().any(|line| line.trim() == entry || line.trim() == ".deciduous") {
            // Already in gitignore
            return Ok(());
        }

        // Append
        let new_content = format!("{}\n\n# Deciduous database (local)\n{}\n", existing.trim_end(), entry);
        fs::write(&gitignore_path, new_content)
            .map_err(|e| format!("Could not update .gitignore: {}", e))?;
        println!("   {} .gitignore (added .deciduous/)", "Updated".green());
    } else {
        // Create new .gitignore
        let content = format!("# Deciduous database (local)\n{}\n", entry);
        fs::write(&gitignore_path, content)
            .map_err(|e| format!("Could not create .gitignore: {}", e))?;
        println!("   {} .gitignore", "Creating".green());
    }

    Ok(())
}
