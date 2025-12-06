---
description: Recover context from decision graph and recent activity - USE THIS ON SESSION START
allowed-tools: Bash(losselot:*, make:*, git:*, cat:*, tail:*)
argument-hint: [focus-area]
---

# Context Recovery

**IMPORTANT**: Run this command at the start of every session or after context compaction to understand the current state of development.

## Automatic Context Gathering

Execute these commands to recover project state:

### 1. Recent Decision Graph Activity
```bash
# Get all decision nodes with descriptions
./target/release/losselot db nodes

# Get recent edges showing decision flow
./target/release/losselot db edges

# Get command log (recent operations)
./target/release/losselot db commands
```

### 2. Git State
```bash
# Current branch and status
git status

# Recent commits (what was just worked on)
git log --oneline -10

# Any uncommitted changes
git diff --stat
```

### 3. Recent Activity Log
```bash
# Check git.log for session history
cat git.log | tail -30
```

## Focus Areas

If $ARGUMENTS specifies a focus area, prioritize context for that topic:

- **lofi** or **cfcc**: Query nodes related to lo-fi detection, CFCC algorithm
- **spectral**: Query spectral analysis nodes and observations
- **ui** or **graph**: Focus on UI/graph viewer state
- **detection**: General detection algorithm decisions

## What to Report Back

After gathering context, summarize:

1. **Current branch** and any pending changes
2. **Recent decisions** from the graph (especially pending/active ones)
3. **Last actions taken** based on git log and command log
4. **Open questions** or observations that haven't been resolved
5. **Next logical steps** based on the decision graph state

## Example Output

```
=== CONTEXT RECOVERED ===

Branch: main (up to date)
Last commit: feat: Add SQLite decision graph system

Recent Decisions:
- [PENDING] Lo-fi detection approach → chose CFCC over Temporal Variance
- [PENDING] Code organization: consider splitting large files

Recent Actions:
- Implemented CFCC in commit aa464b6
- Added decision graph web UI

Open Observations:
- MP3 vs Tape: brick-wall vs gradual rolloff distinction
- Mixed-source detection not yet implemented

Suggested Next Steps:
1. Mark completed decisions as 'completed'
2. Consider mixed-source detection feature
3. Split large files if needed
```

## The Memory Loop

```
SESSION START → Query graph → See past decisions
     ↓
DO WORK → Log observations, decisions, actions
     ↓
BEFORE PUSH → make sync-graph → Export to JSON
     ↓
PUSH → GitHub Pages updates with live graph
     ↓
SESSION END → Graph persists
     ↓
(repeat)
```

**Live graph**: https://notactuallytreyanastasio.github.io/losselot/demo/

## Why This Matters

Context loss during compaction can lead to:
- Repeating work that was already done
- Missing important decisions that were made
- Losing track of the reasoning behind implementations

The decision graph is specifically designed to survive context loss. Query it early, query it often.

## Quick Logging During Session

```bash
make obs T="Something I noticed"
make decision T="Choice I'm making"
make action T="What I implemented"
make link FROM=X TO=Y REASON="why"
```

Then before pushing: `make sync-graph`
