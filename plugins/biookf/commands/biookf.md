---
description: BioOKF quickstart, orient on the bokf tools and open the Studio GUI
---

You are working with **BioOKF** (Biomedical Open Knowledge Format). The `biookf`
MCP server is connected and exposes two tool families:

- **Curation & analysis** (`bokf_*`): `bokf_scaffold`, `bokf_convert` (ingest a
  PDF/HTML/text source), `bokf_write_page` / `bokf_validate_page`, `bokf_lint`,
  `bokf_verify` (the accountability gate), `bokf_search`, `bokf_stats`,
  `bokf_graph`, `bokf_index`, `bokf_merge_raw` / `bokf_merge_snapshot`,
  `bokf_list_bases`, `bokf_set_active` / `bokf_get_active`, `bokf_predicates`
  (the controlled vocabulary: node types, predicates, knowledge levels, agent types).
- **Live Studio control** (`bokf_studio_*`): launch and drive the BioOKF Studio
  desktop app via `bokf_studio_open`, `bokf_studio_status`,
  `bokf_studio_state` (the GUI's full status as JSON, so **read this instead of
  taking screenshots**), `bokf_studio_graph`, `bokf_studio_select`,
  `bokf_studio_search`, `bokf_studio_reload`, `bokf_studio_narrate`,
  `bokf_studio_screenshot`, `bokf_studio_close`.

To get going:
1. Call `bokf_studio_open` to launch the Studio (it auto-installs the prebuilt app
   on first use, with no compile).
2. List registered bundles with `bokf_list_bases` (or use the Studio sidebar). A
   knowledge base can live **anywhere on disk**; register one with `bokf register`
   or the Studio's **+ New base** button.
3. Use `bokf_studio_state` to know what the app is showing, then drive the graph
   with `bokf_studio_select` / `bokf_studio_search`. Each action you take is shown
   live in the app's "AI agent" banner so the user can watch you explore.

$ARGUMENTS
