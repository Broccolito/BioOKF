# BioOKF Studio Graph View — Performance Optimization for Large Bundles

**Date**: 2026-06-28
**Context**: Studio graph view became unusably slow with `benchmark-biored` (2,550 nodes / 5,360 edges).
**Source file**: `app/studio/dist/app.js` (vanilla JS Canvas renderer, no build system, 1,064 → 1,133 lines)

---

## Problem

The graph renderer used a hand-rolled O(n²) n-body force simulation running on the UI thread every animation frame (60 fps). For 2,550 nodes this meant:

- **~3.25 million** pairwise distance calculations per frame
- **~975 million** calculations during the 300-iteration blocking pre-simulation on load
- All 5,360 edges and 2,550 nodes drawn every frame regardless of viewport
- Neighbor sets rebuilt by scanning all edges on every mousemove

This caused multi-second freezes on load and single-digit FPS during interaction.

## Root Causes

| Bottleneck | Location in `app.js` | Cost |
|---|---|---|
| O(n²) pairwise repulsion in `tick()` | Lines ~90–99 | 3.25M distance calcs / frame |
| 300 blocking pre-sim iterations in `loadGraph()` | Line ~103 | 975M calcs before first paint |
| All edges/nodes drawn every frame in `draw()` | Lines ~128–141 | 7,910 draw calls / frame |
| Neighbor scan on every mousemove in `neighborsOf()` | Line ~119 | 5,360 edge iterations / event |
| Force sim never stops in `loop()` | Line ~242 | Wasted CPU after layout settles |

## Changes Applied

### 1. Barnes-Hut Quadtree (O(n²) → O(n log n))

Added a `QNode` class implementing the Barnes-Hut algorithm. Each tick:
1. Build a quadtree from all node positions
2. For each node, traverse the tree — approximate distant clusters with their center of mass (θ = 0.9)
3. Complexity drops from 3.25M to ~28K operations per tick (**~113× speedup**)

```js
// Key constants
const BH_THETA = 0.9;  // opening angle: lower = more accurate but slower
const REP_STRENGTH = 4200;  // repulsion strength (unchanged from original)
```

### 2. Reduced Pre-Simulation (300 → 20 iterations)

The initial circle-layout + 20 quadtree-accelerated iterations provide a sufficient warm-start. The animation loop handles the rest via `requestAnimationFrame`.

### 3. Viewport Culling in `draw()`

Compute visible world-space rectangle from the current pan/zoom transform. Skip nodes and edges whose positions fall outside the viewport + 80px padding. Typical reduction: 90%+ fewer draw calls at normal zoom.

### 4. Energy-Based Auto-Settle

After each tick, compute total kinetic energy (Σ vx² + vy²). When energy stays below `nodes.length × 0.008` for 30 consecutive frames, set `alpha = 0` to stop the simulation entirely — no more CPU usage.

### 5. Auto-Resume on Interaction

Dragging a node or zooming resumes the simulation (`alpha = 1` or `0.25` respectively), then it settles again.

### 6. Pre-Indexed Neighbor Map

Neighbor sets are built once in `loadGraph()` and stored in a `Map<string, Set<string>>`. `neighborsOf()` becomes O(1) instead of scanning all edges.

## File diff summary

- `app/studio/dist/app.js`: 1,064 → 1,133 lines (+69 net)
- `app/studio/dist/app.js.backup`: original preserved
- All changes are in the single `app.js` file — no new dependencies

## Performance Impact (estimated)

| Metric | Before | After |
|---|---|---|
| Force sim per frame | ~3.25M pairwise calcs | ~28K tree traversals |
| Load-time calculations | ~975M (blocking) | ~5M (non-blocking) |
| Draw calls (normal zoom) | 7,910 / frame | ~200–500 / frame |
| Neighbor lookup | O(e) = 5,360 iterations | O(1) = Map.get() |
| Idle CPU after settle | Continuous sim | Zero (frozen) |

## How to verify

1. Start BioOKF Studio pointing at the right root directory
2. Select `benchmark-biored` from the sidebar
3. Graph should appear within 1–2 seconds (was 5+ seconds before)
4. Pan via click-drag on empty space — should feel smooth
5. Zoom with scroll wheel — should be responsive
6. Click a node — detail panel should open instantly
7. Search — node filtering should be real-time
8. After 2–3 seconds of no interaction, the graph should freeze (no jitter)

To revert: `cp app/studio/dist/app.js.backup app/studio/dist/app.js`

## Notes for future

- The quadtree θ parameter can be tuned: lower = more accurate layout but more CPU. 0.9 is a good balance.
- The energy settle threshold (`nodes.length × 0.008`) may need adjustment for different graph topologies.
- If a build system (npm/esbuild) is added later, switching to `d3-force` would give battle-tested Barnes-Hut for free.
- For bundles >10K nodes, additional optimizations may be needed (Web Worker for force sim, WebGL rendering).
