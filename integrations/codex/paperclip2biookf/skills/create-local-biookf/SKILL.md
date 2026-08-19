---
name: create-local-biookf
description: Create and register a verified BioOKF knowledge base from a local folder of PDF, DOCX, Markdown, or text papers using the user's local Codex or Claude subscription. Use for private paper collections and offline folder-to-KB ingestion.
---

# Create a BioOKF KB from local papers

1. Run `bokf connections` and require the selected subscription agent.
2. Confirm the source folder, KB name, workspace, provider, and maximum files.
3. Run `bokf create-local "PAPERS_FOLDER" --workspace "WORKSPACE" --name "NAME" --provider claude --max-files 100 --json`.
4. The workflow converts supported files, extracts schema-bound concepts and evidence, verifies the bundle, registers it in Studio, and retains failed-document diagnostics.
5. Report the bundle path, successful and failed document counts, and verification outcome.

Do not send documents to API endpoints or request API keys. The selected local CLI owns the authenticated subscription session.
