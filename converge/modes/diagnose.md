# Mode: Diagnose

Loaded when `--mode diagnose` is dispatched. Converges on the root cause of a bug or unexpected behavior. Always uses debate mechanism (per `reference/when-debate-helps.md`).

## Workflow

1. **Gather context** — read error messages, logs, relevant code. Include only code paths relevant to the candidate hypotheses (keep reviewer prompts under ~30K tokens). Ask the user for reproduction steps if unclear (per Input validation in SKILL.md — proceed at lower confidence if unavailable).
2. **Generate 2-4 candidate root-cause hypotheses.** For each: supporting evidence + at least one **disconfirming check** (an observation that, if seen, would refute the hypothesis). Single-hypothesis framing anchors both reviewers on the first guess; multiple hypotheses force a comparison.
3. **Round 1 — hypothesis triage.** Both reviewers evaluate ALL hypotheses, score by evidence weight, and propose the disconfirming experiments most worth running. Synthesize.
4. **Prune** hypotheses that fail their disconfirming checks (or are clearly dominated by a sibling). If only one survives, run rounds 2-N convergence on that hypothesis as in standard round structure (SKILL.md). If multiple survive, continue the round-2 cross-critique over the surviving set; only run the final convergence loop after one hypothesis is dominant.
5. **Propose fix to user** — present the fix with evidence once converged on a single root cause. The user decides whether to apply it. Do NOT apply fixes to production code without user confirmation in diagnose mode.
6. **Verification round** — after the user approves and the fix is applied, run one more reviewer pass: "Does this fix address the root cause? Any regressions?" This is a bonus round outside the convergence loop — it does not count toward `--rounds`.

## Mode-specific convergence requirement

Both reviewers must agree on the root cause. Disagreement on cause = not converged, even if no "new" findings are surfaced.

## Calibration interaction

Verbosity-consensus check (`reference/calibration.md` §I3) applies in diagnose mode: if both reviewers prefer the more elaborate root-cause story over the simpler one without new factual support, flag for orchestrator review.
