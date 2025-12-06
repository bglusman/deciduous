---
layout: default
title: Decision Graph
---

# Decision Graph

The decision graph is not documentation written after the fact. It's a real-time record of how this project evolved - captured as decisions were made.

**[Explore the Interactive Graph](demo/)** - see all 56+ nodes, pan/zoom, click to inspect

---

## What is a Decision Graph?

Every software project involves hundreds of decisions: which library to use, how to structure code, which approach to take for a feature. Most of this gets lost. The decision graph captures it all.

The graph is stored in a SQLite database (`losselot.db`) that persists across sessions. When context gets lost (session ends, Claude compacts memory), the graph remains.

---

## Node Types

| Type | Purpose | Example |
|------|---------|---------|
| **Goal** | High-level objectives | "Test lo-fi detection on charlie.flac" |
| **Decision** | Choice points with options | "How to distinguish MP3 cutoff from tape rolloff?" |
| **Option** | Approaches considered | "Approach A: Temporal Cutoff Variance" |
| **Action** | What we implemented | "Implemented CFCC in commit aa464b6" |
| **Outcome** | Results of actions | "CFCC passes 157 tests, detects 25/29 transcodes" |
| **Observation** | Technical insights | "MP3 has brick-wall cutoff, tape has gradual rolloff" |

---

## Edge Types

Edges show relationships between nodes:

- **leads_to** - Natural progression (goal → decision → action)
- **chosen** - We picked this option (and here's why)
- **rejected** - We didn't pick this (and here's why)
- **requires** - Dependencies between nodes
- **blocks / enables** - Impediments or enablers

---

## Real Example: Lo-Fi Detection

This is an actual decision chain from the project.

### The Problem

File `charlie.flac` was flagged as a transcode, but it was actually a legitimate lo-fi recording. The high-frequency rolloff was from tape, not MP3 compression.

### The Decision

**Node 2**: "Lo-fi detection approach"

How do we distinguish MP3 brick-wall cutoff from natural tape/lo-fi rolloff?

### Options Considered

**Node 3 - Approach A**: Temporal Cutoff Variance
- Measure how cutoff frequency varies over time
- MP3 = fixed cutoff, Tape = varies with dynamics
- *Status: Rejected - more complex, requires per-window tracking*

**Node 4 - Approach B**: Cross-Frequency Coherence (CFCC)
- Measure correlation between adjacent frequency bands
- MP3 = sudden decorrelation at cliff
- Tape = gradual decorrelation following the music
- *Status: Chosen - works with existing FFT structure*

### The Implementation

**Node 7**: Implemented CFCC in commit aa464b6
- Added CrossFrequencyCoherence struct
- Cliff detection at known codec cutoffs
- Scoring: +25 for cfcc_cliff, -15 for lofi_safe_natural_rolloff

### The Outcome

**Node 8**: CFCC passes 157 tests, detects 25/29 transcodes

The graph captures why CFCC was chosen over temporal variance - not just what was implemented.

---

## Why This Matters

### Traditional Documentation

```
CHANGELOG.md:
- Added CFCC detection for lo-fi files
```

This tells you *what* changed. Not *why*, not *what else was considered*.

### Decision Graph

Six months from now, when someone asks "why didn't you use temporal variance?", the answer is queryable:

```bash
./losselot db nodes | grep -i temporal
# Node 3: Approach A: Temporal Cutoff Variance
# Node 5: Approach A requires per-window cutoff detection

./losselot db edges | grep "3"
# Edge 7: 2 -> 3 (rejected) "More complex, requires per-window tracking"
```

---

## Query the Graph

```bash
# List all nodes
./losselot db nodes

# Show relationships
./losselot db edges

# Export full graph as JSON
./losselot db graph

# Add observations as you work
./losselot db add-node -t observation "Your finding here"

# Connect nodes with rationale
./losselot db add-edge FROM_ID TO_ID -r "Why they connect"
```

Or use Makefile shortcuts:

```bash
make obs T="Your observation"
make decision T="Your decision"
make action T="What you did"
make outcome T="Result"
make link FROM=1 TO=2 REASON="why"
```

---

## Live Graph Statistics

The graph currently contains:

- **56+ nodes** across 6 types
- **47+ edges** showing relationships
- **4 major goal chains**: lo-fi detection, GitHub Pages site, WASM analyzer, docs fixes

Every node has timestamps, optional descriptions, and status tracking.

---

## The Meta Layer

The decision to *build this decision graph system* is itself documented in the graph:

- **Node 13**: Goal - Create GitHub Pages living museum site
- **Node 14**: Decision - Site structure and content organization
- **Node 16**: Observation - Four pillars of value

The system documents itself. Reading this page means reading decisions tracked in the very system being described.

---

## Known Codec Cutoffs

One observation from the graph (Node 12):

| Bitrate | Cutoff Frequency |
|---------|------------------|
| 64-96 kbps | 10.5-12 kHz |
| 128 kbps | 14-16.5 kHz |
| 192 kbps | 16.5-18.5 kHz |
| 256 kbps | 18-19.5 kHz |
| 320 kbps | 19.5-21 kHz |

These frequencies are used to match detected cliffs to likely source bitrates.

---

[Back to Home](/) | [Try the Analyzer](analyzer.html) | [Claude Tooling](claude-tooling)
