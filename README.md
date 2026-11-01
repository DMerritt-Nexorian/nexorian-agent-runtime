# Nexorian Agent Runtime

Lightweight `no_std` agent that routes tasks to the commercial tool suite.

Local inference only. No external providers.

## Existing tools
- content_create
- write_draft
- language_practice
- task_break
- feedback_analyze

## Next five income tools
- contract_review — commercial contract and clause review
- code_audit — security and quality review of source
- proposal_write — business and sales proposals
- meeting_summary — structured meeting notes
- support_reply — customer support draft responses

## Usage
Implement `LocalLlmEngine` against your local OpenAI-compatible endpoint (or future Nexorian model).
Construct `ToolAgent::new(engine)` and call `step(user_input)`.

Tools currently return queued status. Wire each to its product repository for full execution.
