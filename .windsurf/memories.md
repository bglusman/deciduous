# Deciduous Project Memories

<memory>
This project uses Deciduous for decision graph tracking. The CLI is `deciduous` (installed via cargo).
Key commands: `deciduous add`, `deciduous link`, `deciduous nodes`, `deciduous edges`, `deciduous sync`.
</memory>

<memory>
Decision nodes have optional metadata flags:
- `-c, --confidence <0-100>` for confidence level
- `-p, --prompt "..."` to store the triggering user prompt
- `-f, --files "a.rs,b.rs"` to associate files with the node
- `--commit <hash>` to link to a git commit
</memory>

<memory>
The decision graph is published live at https://notactuallytreyanastasio.github.io/deciduous/
Run `deciduous sync` before pushing to update the live graph.
</memory>

<memory>
Node types: goal, decision, option, action, outcome, observation.
Edge types: leads_to (default), chosen, rejected, requires, blocks, enables.
Always include a reason when rejecting an option!
</memory>

<memory>
LOG BEFORE YOU CODE, NOT AFTER.
SYNC BEFORE YOU PUSH.
</memory>
