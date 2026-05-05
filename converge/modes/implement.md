# Mode: Implement

Loaded when `--mode implement` is dispatched. Converges on a full implementation of a spec or plan. The orchestrator (Claude) writes the code; reviewers verify each step against the spec.

## Workflow

1. **Read the spec and impl plan** — identify the ordered list of steps/stories.
2. **For each step:**
   a. **Implement** — write the code changes for this step.
   b. **Self-check** — run typecheck, lint, tests. Fix any failures.
   c. **Launch both reviewers** — send them the **review packet for this step only** (not full files, not cumulative diffs). The review packet contains:
      - (i) the step diff,
      - (ii) full bodies of any function modified in the diff,
      - (iii) full bodies of helpers called from those functions (one level deep),
      - (iv) type / struct / interface definitions referenced in the diff,
      - (v) the initialization paths from prior steps if this diff mutates shared state.
      Wrap each block with a source-span tag inside the `UNTRUSTED TARGET` wrapper (the normative definition for both is in `reference/threat-model.md` > Untrusted-target wrapping). Reviewers may issue ONE context-request per round (e.g., "show me caller X" or "show me state Y after step N-1"); the orchestrator must answer before that round's findings are accepted. The 30-line-of-context heuristic is insufficient for the aliasing and initialization-ordering bugs Rules 10-11 explicitly target — those bugs typically span untouched callers, helpers in other files, and prior init steps.
   d. **Run rounds 1-N** per the shared round structure in SKILL.md.
   e. **Step converged** → move to next step.
3. **Final verification** — after all steps, run both reviewers on the full changeset vs. the spec: "Is the spec fully implemented? Any gaps?" This is a single pass, not a convergence loop. If final verification finds High/Medium issues, create a follow-up implementation step to address them, then rerun final verification. Repeat until clean or user stops.
4. **Report.**

## Mode-specific convergence requirement

Per-step convergence. Each step must satisfy the criteria independently. Final verification round on full changeset does not count toward step rounds.

## Mechanism gating (per `reference/when-debate-helps.md`)

- **Steps touching shared mutable state, async/concurrency, security boundaries, public APIs, data migrations, on-disk format changes** → debate.
- **Pure refactors with full test coverage, type-checker-covered signature changes, trivial additions** → orchestrator-auto-selected self-consistency (`mech: auto-self-consistency K=3`). NOT the same as user-chosen `--single-reviewer`. The auto path escalates to fresh blind Codex debate when disagreement ≥30%; the user-chosen flag does not. See `reference/when-debate-helps.md` and SKILL.md > Self-consistency structure > step 3 for full dispatch semantics. Saves roughly 50% on routine steps.

## Key rules

- **You write the code, reviewers verify.** Don't delegate implementation to subagents.
- **Typecheck/lint/test between steps.** Don't accumulate broken code.
- **NEVER skip reviewer rounds.** Every step must be reviewed before committing. Do not commit steps while "waiting for reviewers on a previous step." Skipping rounds to move fast is false economy — bugs that slip through cost more time than the review.
- **Commit after each converged step** (if the user wants — ask on the first step, then follow that preference). Stage only files modified in the current step by name (not `git add -A`). Use message format: `converge: step N — [step name]`. If not in a git repo, skip commits. If a pre-commit hook fails, fix the issue and create a new commit.
- **If a reviewer finding requires changing the spec or plan**, flag it to the user before proceeding. Don't silently deviate from the spec.
- **If stuck on a step** (reviewers keep finding new issues after max rounds), pause and ask the user.

## `accept risk: F# [reason]` semantics

Implement mode allows unresolved Medium findings to be bypassed via `accept risk: F# [reason]`, but only after a Round 2.5 adjudication pass on F# (see `reference/calibration.md` §I5 and SKILL.md > Round 2.5 Adjudication). If the adjudicator returns `valid` and the user still wants to proceed, the finding is recorded as `[user-accepted-risk: <reason>]` in the report alongside the adjudicator's verdict. Unresolved High findings BLOCK skip; the user must `stop` or `override F# [reason]`.
