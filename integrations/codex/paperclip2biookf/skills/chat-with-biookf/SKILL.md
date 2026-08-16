---
name: chat-with-biookf
description: Answer questions from a selected local BioOKF knowledge base using grounded node-and-edge retrieval and the user's local Codex or Claude subscription. Use when asked to chat with, interrogate, summarize, compare, or trace provenance inside a BioOKF KB.
---

# Chat with a BioOKF knowledge base

Use `bokf chat` so answers are grounded in retrieved BioOKF nodes and provenance.

1. Run `bokf connections`; require the selected subscription agent.
2. Resolve the exact KB path. If ambiguous, use `bokf register --list` and ask the user to choose.
3. Run `bokf chat "BUNDLE_PATH" "QUESTION" --provider codex` (or `claude`). Omit `--model` for the subscription default.
4. Present the answer with its node identifiers and evidence URLs. Clearly distinguish explicit evidence, associations, predictions, contradictions, and missing evidence.

Do not augment the answer with web knowledge unless the user separately requests external research. Do not request or use API keys.
