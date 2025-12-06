# Sync Decision Graph to GitHub Pages

Export the current decision graph to docs/demo/graph-data.json so it's deployed to GitHub Pages.

## Steps

1. Run `make sync-graph` to export the graph
2. Show the user how many nodes/edges were exported
3. If there are changes, ask if they want you to stage them

This should be run before any push to main to ensure the live site has the latest decisions.
