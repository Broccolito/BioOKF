---
name: chat-with-biookf
description: Chat with a selected local BioOKF knowledge base using grounded retrieval and a local Codex or Claude subscription. Use for questions, summaries, comparisons, and provenance tracing inside a KB.
---

# Chat with a BioOKF KB

Run `bokf connections`, resolve the exact bundle path, then run `bokf chat "BUNDLE_PATH" "QUESTION" --provider claude`. Omit `--model` for the subscription default. Return node identifiers and evidence URLs, separating evidence, association, prediction, contradiction, and missing evidence. Do not add external knowledge or request API keys.
