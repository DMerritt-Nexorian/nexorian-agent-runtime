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
- contract_review
- code_audit
- proposal_write
- meeting_summary
- support_reply

## Simulation
A `SimulatedEngine` in `src/lib.rs` tests drives the loop:
1. First generate returns a `call_tool` for `feedback_analyze`.
2. Agent executes the tool and appends the result.
3. Second generate returns `final_response`.
4. `step` returns the content.

The original example called `query_customer_account`. That tool is not registered. Use one of the ten names above.

## What still needs wiring
- Tool `execute` methods currently return status JSON. Connect each to its product repository for real work.
- No real model is attached. `LocalLlmEngine` must be implemented against a local endpoint or future Nexorian model.
- Price IDs for the five new catalog items are empty until Stripe products are created.
