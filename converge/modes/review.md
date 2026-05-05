# Mode: Review

Loaded when `--mode review` is dispatched. Converges on the quality of a document (spec, plan, design doc, code file).

Reviewers receive: the target document + any `--focus` context. **Context budget:** for targets over 500 lines, send a summary + the sections most relevant to `--focus` (or the full doc if no focus is specified and it fits). Target: reviewer prompt should not exceed ~30K tokens including boilerplate and accumulated findings.

## Source-span tagging (required when sending excerpts)

Use the wrapper format defined normatively in `reference/threat-model.md` > Untrusted-target wrapping (code/spec/doc-source variant). Every excerpt or summary block must be wrapped so reviewer evidence remains traceable to the source artifact.

Reviewers cite excerpts by quote + source-span tag (e.g., "src/foo.ts L1200-L1290, the line `return foo()`"). Bare excerpt-relative line numbers are not accepted as evidence (see Rule 1 of the reviewer block in SKILL.md).

## Fix application policy

Auto-apply ONLY non-semantic clarifications: typos, grammar, broken cross-references, formatting, dead links. Any change that alters requirements, scope, acceptance criteria, architectural decisions, public API surface, named invariants, or numeric thresholds requires explicit user approval before mutation. Findings that imply a semantic change are flagged `blocked-on-user` and the round continues without applying the edit. This prevents review mode from collapsing into unauthorized authorship — the orchestrator must not silently rewrite the spec under review and then treat the rewrite as ground truth in subsequent rounds. Implement mode enforces an analogous rule ("If a reviewer finding requires changing the spec or plan, flag it to the user before proceeding"); review mode matches.

Both reviewers see the updated document in subsequent rounds. **For source code targets:** run typecheck/lint/tests after applying fixes, same as implement mode. If they fail, revert the fix and flag it as disputed.
