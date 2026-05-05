# Agreement-vs-correctness calibration

Loaded by SKILL.md unconditionally. Defines orchestrator-side checks that run during round-2 synthesis BEFORE applying any agreed fixes, plus the `accept risk` and `override F#` semantics that route findings into Round 2.5 adjudication.

Reviewer agreement is a useful signal only when judges are blind, order-randomized, independently prompted, and calibration-checked. Otherwise, "both agree" can mean "both share a bias" (Zheng et al., "MT-Bench," 2023 — https://arxiv.org/abs/2306.05685; Thakur et al., "Judging the Judges," 2024 — https://arxiv.org/abs/2406.12624; Wataoka et al., "Self-Preference Bias," 2024 — https://arxiv.org/abs/2410.21819).

## §I1 Surface-heuristic consensus check

Run during round-2 synthesis, BEFORE applying any agreed fixes. If both reviewers' rationales for an agreed finding cite the exact same source span AND rationale token-overlap exceeds 50%, route the finding to Round 2.5 adjudication.

- **Tokenizer**: lowercase, split on `\W+`, drop tokens of length ≤2, drop the standard English stopword set, drop any token appearing inside the cited span quote.
- **Score**: Jaccard = |A ∩ B| / |A ∪ B|.
- **Cost**: orchestrator-side, runs once per agreed finding, costs no model calls.

Sources: Song et al. 2026 "Evaluation Illusion" — https://arxiv.org/abs/2603.11027; Thakur et al. 2024.

## §I2 Order-swap probe (high-stakes only)

Triggered for `--severity high` runs OR before adjudicating a disputed finding. The round-2 flow already sends each reviewer the OTHER's findings in a randomized internal order; the order-swap probe re-runs ONE reviewer's round-2 pass with the internal order reversed (last finding shown first). If the verdict on any finding flips between the two orderings, that finding is order-dependent → route to adjudication. Probe runs on one reviewer only (cost-bounded); flips on either side are sufficient signal.

Sources: Zheng et al. MT-Bench (2023); Shi et al. (2024) — https://arxiv.org/abs/2406.07791.

## §I3 Verbosity-consensus check (write + diagnose mode)

If both reviewers prefer the more verbose alternative (a longer rewrite, a longer hypothesis explanation) AND their justifications cite "more thorough" / "more detailed" / "more complete" without new factual support, flag for orchestrator review. In diagnose mode, this manifests as preferring the more elaborate root-cause story over the simpler one. **In write mode**, applied during Phase 1 (Accuracy) synthesis when both reviewers raise an "incomplete" finding with overlapping rationale but no new factual gap; route the finding to orchestrator review rather than the spec author. Write mode has no Phase 3 cross-critique by construction (per `modes/write.md`), so §I3 does not apply there.

Sources: Saito et al. (2023) — https://arxiv.org/abs/2310.10076; Zheng et al. (2023); Thakur et al. (2024).

## §I4 Self-style guard (built-in via cross-family reviewers)

Critic = Claude, Codex = GPT-5.5 satisfies this by construction. Keep it that way. If `--single-reviewer` is set, the self-style check is unavailable; the run is flagged `[self-style: unverified]` in the report. If the Round 2.5 judge falls back to the same family as Codex (no third family available), the run is flagged `[adjudicator-family: same as Codex — self-style guard partial]`.

Source: Wataoka et al. 2024.

## §I5 Rebuttal-driven sycophancy guard (`override F#` semantics)

When the user invokes `override F#`, the adjudicator (Round 2.5 judge) sees the user's reason as new evidence to evaluate, not as a directive. The user's reason is appended to the OPPONENT side as one labeled `[user-override-reason]` evidence item; the judge's prompt explicitly forbids agreeing with the user without independent target evidence. The opponent role does NOT translate the user's framing into stronger language; it relays verbatim. This codifies the existing `override F#` semantics as a sycophancy mitigation.

`accept risk: F# [reason]` (implement mode, Medium findings only) ALSO triggers a single Round 2.5 adjudication pass before the step advances. If the adjudicator returns `valid` and the user still wants to proceed, the finding is recorded as `[user-accepted-risk: <reason>]` in the report alongside the adjudicator's verdict. This prevents `accept risk` from becoming a frictionless sycophancy bypass.

Sources: OpenAI sycophancy postmortem (May 2025) — https://openai.com/index/expanding-on-sycophancy/; Sharma et al. (2023) — https://arxiv.org/abs/2310.13548; Kim & Khashabi (2025) — https://arxiv.org/abs/2509.16533.

## §I6 Joint-agreement floor

Never accept a Round 2 finding solely on 2/2 reviewer agreement when the finding (a) lacks two independent target quotes (one from each reviewer), AND (b) was raised by only one reviewer in round 1. Such findings get a Round 2.5 judge pass even if both reviewers ostensibly "agreed" in cross-critique. Cheap insurance against shared-bias confirmation.

Sources: Xu et al. (2026) — https://arxiv.org/abs/2604.06820; Jung et al. (2024) — https://arxiv.org/abs/2407.18370.

---

These checks are orchestrator-side; they add at most one judge call per flagged finding (bounded by the agreed-set size). They do NOT replace the standard round structure. The progress signal field `Calib-flagged: C` reports the count of findings routed through these checks per round.

## Citations

- Zheng et al., MT-Bench (2023): https://arxiv.org/abs/2306.05685
- Thakur et al., Judging the Judges (2024): https://arxiv.org/abs/2406.12624
- Wataoka et al., Self-Preference Bias (2024): https://arxiv.org/abs/2410.21819
- Song et al., Evaluation Illusion (2026): https://arxiv.org/abs/2603.11027
- Shi et al. (2024): https://arxiv.org/abs/2406.07791
- Saito et al. (2023): https://arxiv.org/abs/2310.10076
- OpenAI Sycophancy postmortem (May 2025): https://openai.com/index/expanding-on-sycophancy/
- Sharma et al. (2023): https://arxiv.org/abs/2310.13548
- Kim & Khashabi (2025): https://arxiv.org/abs/2509.16533
- Xu et al. (2026): https://arxiv.org/abs/2604.06820
- Jung et al. (2024): https://arxiv.org/abs/2407.18370
