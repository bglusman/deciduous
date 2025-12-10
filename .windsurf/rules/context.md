---
description: Recover context from decision graph on session start - run deciduous nodes and edges to see past decisions
globs:
alwaysApply: false
---

<context_recovery>

# Context Recovery

**Use this at session start to recover from context loss.**

## Quick Context Commands

<commands>
```bash
# See all decisions
deciduous nodes

# See connections
deciduous edges

# Recent command history
deciduous commands

# Git state
git status
git log --oneline -10
```
</commands>

## After Recovery

<post_recovery>
1. Identify pending/active decisions
2. Note any unresolved observations
3. Check for incomplete action→outcome chains
4. Resume work on the most relevant goal
</post_recovery>

## Remember

<reminder>
The graph survives context compaction. Query it early, query it often.
Log decisions IN REAL-TIME as you work, not retroactively.
</reminder>

</context_recovery>
