# BioOKF Studio — UI overhaul, node editing, citation previews, chrome fix

**Date:** 2026-06-27

## Goal
Overhaul the BioOKF Studio frontend to a Codex/ChatGPT aesthetic, add per-node
markdown editing (persisted to disk), make the existing blue-grey citation
references clickable to a read-only preview, and fix the macOS traffic-light
overlap — without disturbing existing backend logic, the data-loading flow, or
current features.

## Constraints
- Reuse existing structure; **preserve all DOM IDs/classes** and the
  `window.__okf` API + `window.__OKF_READY` flag (Playwright tests depend on them).
- Keep changes minimal beyond the requested fixes.
- Responsive.
- Backend changes are permitted but limited to the single additive command
  needed for saving.

## Findings (context)
- Frontend is vanilla `studio/app/dist/index.html` + `app.js`; data comes from
  `dist/data/*.json` via `fetch`. The Tauri commands exist but are dormant
  because `window.__TAURI__` is undefined (`withGlobalTauri` is not set), so the
  app uses the fetch path. **Keep the fetch path** — curated base names live in
  the static `bases.json`; the live `base_info` returns `name = dir-id`, which
  would regress the sidebar names.
- **Traffic-light overlap root cause:** the `.tauri` class is gated on
  `window.__TAURI__` (undefined), so `html.tauri .titlebar{padding-left:82px}`
  never applies. Fix the detection.
- Each `pages[id]` carries `path` (e.g. `knowledge/gene/braf.md`) and `body`.
  Node files are `---`-fenced YAML frontmatter followed by a markdown body.
- **Citations / blue-grey texts:**
  - Node description bodies contain markdown links `[text](../type/slug.md)`
    rendered as blue-grey `.md a` (`rgb(79,90,138)` = `#4f5a8a`), but `inl()`
    strips the href → dead links (e.g. "insulin resistance" in Type 2 diabetes).
  - Edge `primary_source` renders blue-grey `#4f5a8a` and currently navigates to
    the Publication node.

## Design

### 1. Top-left chrome fix
- Robust desktop detection: `desktop = !!(window.__TAURI__ || window.__TAURI_INTERNALS__)`;
  toggle `.tauri` on `<html>`.
- On desktop, the top bar reserves a ~76px traffic-light "corner" (left padding)
  with a subtle hairline divider; the graph title + lint pill shift right. No
  reserve in a plain browser.

### 2. Codex/ChatGPT redesign (CSS-only, structure preserved)
- Flat, light, neutral palette (white / `#f7f7f8` surfaces, `#0d0d0d`/`#353740`
  text, `#6e6e80` muted, `#ececf1` hairlines, `#10a37f` accent used sparingly).
  Move away from the heavy frosted-glass translucency.
- Refined system-sans type scale and weights; monospace for ids/code.
- 8–10px radii, hairline borders over heavy shadows, light-gray hover fills.
  Restyle sidebar, KB rows, top bar, detail panel, legend, zoom controls, log
  drawer. Keep every class name/ID.

### 3. Node markdown editing (reuse the `.detail` panel)
- Add an Edit affordance to the Document section → a textarea pre-filled with
  `pages[id].body` + Save / Cancel.
- Save → `tauriInvoke('save_node_body', {base, path, body})`; on success update
  in-memory `pages[id].body` and re-render, with a saved/error state. Desktop
  only; degrades gracefully in the browser (Save disabled with a hint).
- Backend (additive): `save_node_body(base, rel_path, body)` resolves the bundle
  root via the existing `resolve()`, guards path traversal (must stay within the
  bundle root), preserves the YAML frontmatter byte-for-byte (everything up to
  and including the closing `---`), replaces the body, and writes. No existing
  command or okf-core logic changes.
- Frontend invoke helper:
  `tauriInvoke = (cmd,args) => (window.__TAURI__?.core?.invoke || window.__TAURI_INTERNALS__?.invoke)?.(cmd,args)`.
  The data-loading path is unchanged.

### 4. Clickable citation previews (no new sections)
- Node body links: in `inl()`/`renderMd`, keep `[text](href)` blue-grey but
  attach `data-cite="href"` (to be resolved relative to the current page `path`).
  Click → resolve to the target page (normalize the relative path against
  `pages[].path`, fallback to filename match) → open the preview.
- Edge `primary_source`: switch from navigate (`data-node`) to preview
  (`data-cite`) of the Publication page.
- Preview surface: a read-only sheet styled like `.detail` (Codex look) showing
  the target page's title + rendered body (+ a note where the raw source lives),
  with a Back/close that returns to the prior node/edge. Reuses `renderMd` and
  already-loaded page content. No backend.

### 5. Responsive
- Breakpoint (~720–820px): detail panel + log drawer become full-width sheets;
  sidebar auto-collapses; top bar condenses; legend repositions. Canvas is
  already fluid.

## Files
- `studio/app/dist/index.html` — CSS overhaul, top-left corner, responsive,
  small markup for the edit/preview affordances.
- `studio/app/dist/app.js` — desktop detection + invoke helper; edit mode;
  citation linkify + preview sheet. Data-loading unchanged.
- `studio/app/src-tauri/src/main.rs` — add `save_node_body` (additive) + register.

## Out of scope / unchanged
- `tauri.conf.json`, `capabilities/default.json`, okf-core, the existing four
  commands, and the data-loading flow.

## Verification
- Browser mirror (`http://localhost:8754` via Claude Preview) for all
  interaction / visual / responsive checks (identical frontend).
- Rebuild the Tauri app to verify the `.tauri` traffic-light corner and that
  `save_node_body` writes to disk (check file bytes); screenshot via the Tauri
  MCP `take_screenshot`.
- Playwright selectors/API preserved → existing tests pass; refresh the
  committed screenshots.

## Risks
- `window.__TAURI_INTERNALS__.invoke` availability — verify on rebuild; fallback
  is to enable `withGlobalTauri` and special-case base names.
- A Tauri rebuild is required to test the backend command and desktop chrome.
