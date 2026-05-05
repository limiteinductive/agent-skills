# When debate helps: task-shape gating

Loaded by SKILL.md unconditionally. Determines whether a target/step uses two-reviewer debate or single-reviewer self-consistency, BEFORE round 1 launches.

Two-reviewer debate is not uniformly better than cheaper single-reviewer baselines (Smit et al., ICML 2024 — https://proceedings.mlr.press/v235/smit24a.html — show MAD does not reliably outperform self-consistency without task-specific tuning; Kenton et al., 2024 — https://arxiv.org/abs/2407.04622 — find debate's advantage holds primarily under information asymmetry).

## Decision rules

- **Diagnose mode**: ALWAYS debate (information asymmetry between candidate hypotheses).
- **Implement mode steps** touching: shared mutable state, async/concurrency, security boundaries, public APIs, data migrations, on-disk format changes → debate.
- **Implement mode steps** that are pure refactors with full test coverage, type signature changes covered by the type-checker, or trivial additions → **auto-self-consistency, K=3 default** (orchestrator-selected, NOT the same as the user passing `--single-reviewer`). The orchestrator runs the K-sample Critic-only flow internally and labels the run `[mechanism-auto: when-debate-helps]`. Per `SKILL.md` > "Self-consistency structure > step 3 > Auto-selected", high disagreement (≥30%) escalates to a fresh blind Codex debate. The user-chosen `--single-reviewer` flag does NOT escalate (per the same section); the auto path does. Do NOT collapse the two paths to the same `--single-reviewer --self-consistency 3` shorthand: that loses the auto/user provenance the escalation gate depends on. Saves roughly 50% on routine steps.
- **Review mode**: debate by default. Allow user-chosen `--single-reviewer` for low-stakes targets (changelogs, minor doc updates) — explicit user opt-in only, no auto-self-consistency in review mode.
- **Write mode**: per-phase rules in `modes/write.md` override this section.

The orchestrator picks the mechanism BEFORE round 1 launches and reports the choice in the progress line: `mech: debate` / `mech: auto-self-consistency K=k` / `mech: self-consistency K=k` (the last is user-chosen `--single-reviewer`).

## Citations

- Smit et al., MAD evaluation (ICML 2024): https://proceedings.mlr.press/v235/smit24a.html
- Kenton et al., Scalable Oversight (2024): https://arxiv.org/abs/2407.04622
- Du et al., Multiagent Debate (ICML 2024): https://proceedings.mlr.press/v235/du24e.html
- Khan et al., Debate Helps (ICML 2024): https://proceedings.mlr.press/v235/khan24a.html
