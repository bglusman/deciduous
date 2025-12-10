# Deciduous

**Decision graph tooling for AI-assisted development.**

Track every goal, decision, and outcome. Survive context loss. Query your reasoning.

```bash
cargo install deciduous
cd your-project
deciduous init
deciduous serve
```

**[Live Demo](https://notactuallytreyanastasio.github.io/deciduous/)** | **[Browse the Graph](https://notactuallytreyanastasio.github.io/deciduous/graph/)**

---

## The Problem

Claude (and LLMs generally) lose context. Sessions end, memory compacts, decisions evaporate. Six months later, no one remembers *why* you chose approach A over approach B.

## The Solution

Every decision is tracked in a queryable graph that persists forever:

```bash
deciduous add goal "Implement user authentication" -c 90
deciduous add decision "Choose auth method" -c 75
deciduous add option "JWT tokens" -c 80
deciduous add option "Session cookies" -c 70
deciduous link 1 2 -r "Goal requires a decision"
deciduous link 2 3 -r "Option A"
deciduous link 2 4 -r "Option B"
```

View your graph at `localhost:3000`:

```bash
deciduous serve
```

---

## Quick Start

### 1. Install

```bash
cargo install deciduous
```

### 2. Initialize in your project

```bash
cd your-project
deciduous init
```

This creates:
- `.deciduous/` - Database directory
- `.claude/commands/decision.md` - `/decision` slash command
- `.claude/commands/context.md` - `/context` slash command
- `CLAUDE.md` - Project instructions with workflow section
- `.gitignore` - Ignores `.deciduous/`

### 3. Start tracking

```bash
# Add nodes
deciduous add goal "Build feature X" -c 90
deciduous add action "Implementing feature X" -c 85

# Link them
deciduous link 1 2 -r "Goal leads to action"

# View the graph
deciduous serve
```

### 4. Query your history

```bash
deciduous nodes     # List all decisions
deciduous edges     # List all connections
deciduous graph     # Export full JSON
```

---

## Node Types

| Type | Purpose | Example |
|------|---------|---------|
| `goal` | High-level objectives | "Add user authentication" |
| `decision` | Choice points with options | "Choose auth method" |
| `option` | Approaches considered | "Use JWT tokens" |
| `action` | What was implemented | "Added JWT middleware" |
| `outcome` | What happened | "JWT auth working" |
| `observation` | Technical insights | "Existing code uses sessions" |

## Edge Types

| Type | Meaning |
|------|---------|
| `leads_to` | Natural progression |
| `chosen` | Selected option |
| `rejected` | Not selected (include reason!) |
| `requires` | Dependency |
| `blocks` | Preventing progress |
| `enables` | Makes something possible |

---

## Claude Integration

Deciduous creates Claude slash commands automatically:

### `/context` - Session Start

Run this at the start of every session to recover context:

```
/context
```

This queries the decision graph and git state, then reports:
- Recent decisions (especially pending/active ones)
- Current branch and pending changes
- Suggested next steps

### `/decision` - Log Decisions

Log decisions as you work:

```
/decision add goal "Add dark mode"
/decision add action "Implementing theme toggle"
/decision link 1 2 "Goal leads to action"
/decision nodes
```

### The Workflow

```
SESSION START
    |
Run /context -> See past decisions
    |
DO WORK -> Log BEFORE each action
    |
AFTER CHANGES -> Log outcomes
    |
BEFORE PUSH -> deciduous sync
    |
SESSION END -> Graph survives
```

---

## Commands

```bash
# Initialize
deciduous init                          # Set up in current directory

# Add nodes
deciduous add <type> "<title>" [-c CONF] [--commit HASH]
# Types: goal, decision, option, action, outcome, observation

# Link nodes
deciduous link <from> <to> [-r "reason"] [-t edge_type]

# Query
deciduous nodes                         # List all nodes
deciduous edges                         # List all edges
deciduous graph                         # Full JSON export

# Serve
deciduous serve [--port 3000]           # Start web viewer

# Export
deciduous sync [-o path]                # Export to JSON file
deciduous backup [-o path]              # Create database backup

# Status
deciduous status <id> <status>          # Update node status
deciduous commands [-l 20]              # Show recent command log
```

---

## Confidence Weights

Every node can have a confidence score (0-100):

```bash
deciduous add decision "Use CFCC algorithm" -c 85
```

| Range | Meaning |
|-------|---------|
| 90-100 | Certain, proven, tested |
| 70-89 | High confidence, standard approach |
| 50-69 | Moderate, some unknowns |
| 30-49 | Experimental, might change |
| 0-29 | Speculative, likely to revisit |

## Commit Linking

Link decisions to specific code changes:

```bash
deciduous add action "Implemented feature" -c 90 --commit abc123
```

The web viewer shows clickable commit badges that link to GitHub.

---

## Web Viewer

The built-in viewer has 4 tabs:

| Tab | View |
|-----|------|
| **Chains** | BFS chain detection, session grouping, sidebar navigation |
| **Timeline** | Git commits + decisions merged chronologically |
| **Graph** | D3.js force-directed, drag/zoom |
| **DAG** | Dagre hierarchical layout |

Start it with:

```bash
deciduous serve --port 3000
```

---

## GitHub Pages Deployment

Export your graph for static hosting:

```bash
deciduous sync -o docs/graph-data.json
git add docs/graph-data.json
git commit -m "Update decision graph"
git push
```

The web viewer works without a server - it reads the JSON file directly.

---

## The Story

Deciduous was extracted from [losselot](https://github.com/notactuallytreyanastasio/losselot), an audio forensics tool that grew into an experiment in AI-assisted development.

The core insight: Claude sessions lose context, but the decision graph survives. Every goal, every rejected approach, every "why did we do it this way?" is preserved and queryable.

This isn't documentation written after the fact. It's a real-time record of how software gets built - captured as decisions happen, not reconstructed from memory later.

**[Browse the live graph](https://notactuallytreyanastasio.github.io/deciduous/graph/)** to see 100+ decisions from the development process.

---

## Building from Source

```bash
git clone https://github.com/notactuallytreyanastasio/deciduous.git
cd deciduous
cargo build --release
./target/release/deciduous --help
```

### Rebuild the Web Viewer

The React viewer is bundled into the binary. To rebuild:

```bash
cd web
npm install
npm run build:embed
cd ..
cargo build --release
```

---

## License

MIT - Do whatever you want with it.
