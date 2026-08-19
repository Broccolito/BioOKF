---
name: create-local-biookf
description: Create a verified BioOKF knowledge base from a local folder of PDF, DOCX, Markdown, or text papers with a local Codex or Claude subscription. Use for private paper collections and local folder ingestion.
---

# Create from local papers

Run `bokf connections`, then `bokf create-local "PAPERS_FOLDER" --workspace "WORKSPACE" --name "NAME" --provider claude --max-files 100 --json`. Report the registered bundle, verification, and any failed documents. Never request API keys or send papers to an API endpoint.
