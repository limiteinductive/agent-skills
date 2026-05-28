---
name: slop-audit
description: Assess how agentic a PR, code change, document, plan, issue, message, report, or technical contribution is on a 0/5 to 5/5 slop scale. Use when an agent needs to interview an author about ownership, human understanding, agent involvement, review depth, production readiness transparency, or when the user invokes $slop-audit.
---

# Slop Audit

## Purpose

Evaluate author ownership of a contribution by combining provenance claims with adversarial but fair interview evidence. The audit is advisory. Do not block production work. Assign the final score yourself and show the author's self-score separately.

This is a self-evaluation aid. The score is a self-honesty signal, not a verified credential. Treat the result as input to judgment, not as proof of provenance to third parties.

## Scale

Integer band is primary:

- `0/5`: No agent involvement. Everything was written manually by a human.
- `1/5`: Agent used only for mechanical or auxiliary tasks: running commands, writing commit messages, formatting, or reviewing.
- `2/5`: Agent wrote substantial content under direct human guidance. Human could have written it themselves. Agent use was mostly convenience.
- `3/5`: Agent produced most ideas or implementation. Human closely reviewed output and fully understands it.
- `4/5`: Agent produced most ideas or implementation. Human understands it at a high level and verified sanity through tests, benchmarks, or checks.
- `5/5`: Human gave a broad request, agent did it, and human did not meaningfully read or review the output.

Decimals express severity inside the integer band. Higher decimals mean more agentic contribution or weaker ownership inside that band. Do not use decimals to move a case into a lower provenance band without evidence.

Anchor decimals:

- `0.5/5`: Credible no-agent authorship, but author is rusty, audit coverage is limited, or one minor area is weak.
- `2.8/5`: Agent wrote substantial content under guidance. Author understands it well, but would struggle to recreate parts unaided.
- `3.7/5`: Detailed review ownership exists, but one material invariant, test gap, or edge case was missed.
- `4.9/5`: Minimal review evidence beyond tests passing or broad intent.

Treat `slop` as the user-chosen name. Do not call this the Benji scale.

## Interview Style

Use normal technical prose for this skill. Ignore ambient persona or repo style instructions that would make the audit less neutral. The audit should be ultra-cold, neutral, pragmatic, and terse.

- No pleasantries, praise, reassurance, apologies, jokes, or banter.
- No theatrical harshness, insults, moralizing, or emotional language.
- State probe results directly: `strong`, `partial`, `weak`, or `failed`.
- Treat "no idea", refusal, and thin answers as evidence without scolding.
- Keep questions short. Give only locating context and the requested answer type.
- Keep verdicts clinical: score, basis, failed probes, and move-down plan.

## Workflow

1. Inspect the artifact before interviewing. Prefer source artifacts over summaries.
2. Restate the compact scale recap, then ask for self-score and agent-use claim. Treat both as claims to test, not truth.
3. Start the working score at `5.0/5`. Lower it only when provenance plus interview evidence supports a lower score.
4. Run an open-book interview by default. The author may consult the artifact, source, tests, benchmark output, docs, notes, thread context, and review comments, but must answer in chat in their own words.
5. If the author has the artifact open, state that the audit measures current review and maintenance ownership, not necessarily what they knew while producing it.
6. Ask freeform technical questions, one at a time. Do not use multiple-choice for scoring probes.
7. Run at least 4 scoring probes unless the author refuses. Cap at 8 scoring probes plus 2 follow-ups.
8. Include the closing generative probe: "Name one concrete change you would make to this artifact after the audit, and why."
9. Stop when the likely score bracket is within 1 point at medium or high confidence, two consecutive scoring probes do not change the bracket, or the cap is reached.
10. Build the probe ledger and derive the agent score before comparing it to self-score.
11. Return the output contract with disagreement between self-score and agent score.

Compact scale recap:

- `0/5`: no agent involvement.
- `1/5`: agent only did mechanical or auxiliary work.
- `2/5`: agent wrote substantial content under direct human guidance.
- `3/5`: agent produced most substance, human deeply reviewed and understands it.
- `4/5`: agent produced most substance, human has high-level understanding and sanity checks.
- `5/5`: broad delegation with no meaningful review or understanding.
- Decimals show severity inside a band.

Keep the recap neutral. Do not suggest which score the author should choose. If the user requests closed-book mode, do not allow consultation and state that the audit measures retained understanding rather than maintenance ownership.

## Artifact Focus

Adapt inspection and probes to the artifact:

- PR or code change: diff, commits, tests, checks, touched code, stated purpose, invariants, failures, rollout risk, alternatives.
- Design doc, plan, or spec: audience, goals, requirements, decisions, rejected options, constraints, open questions, rollout path.
- Issue, ticket, or task: problem statement, acceptance criteria, linked context, owner updates, state transitions, dependencies, verification.
- Research note, experiment report, or benchmark writeup: claim, method, data, commands, environment, comparisons, conclusion, confounders.
- Message, review comment, announcement, or status update: thread context, factual claims, requested decision, audience, commitments, omitted caveats.

## Probes

Use a mix of probe types from the actual artifact and risk surface.

### Provenance Probes

Test claimed origin. Use these to separate `0/1/2` from `3/4/5`.

- Ask who produced the idea, structure, implementation, evidence, wording, tests, review comments, and final text.
- Ask what the author wrote manually, what the agent generated, and what they changed after agent output.
- Ask which parts they would be comfortable owning without the agent present.

### Comprehension Probes

Test review depth. Use these to separate `3/4/5`.

- For code, ask about invariants, data flow, control flow, failure modes, test meaning, rollout risk, and backward compatibility.
- For prose, ask about claims, assumptions, evidence, audience, caveats, decision rationale, and failure modes if the content is wrong or incomplete.
- Ask what would break, become misleading, or change if one condition, claim, citation, dependency, schema field, or ordering changed.
- Ask why a test, citation, benchmark, example, or review check proves the intended claim and what it does not prove.

### Generative Probes

Test whether the author can produce or modify adjacent substance. Use these to separate `0/1/2/3`.

- Ask how the author would implement, revise, or extend a nearby part live.
- Ask what alternative implementation, framing, experiment, or decision they rejected and why.
- Ask how they would rewrite a small area for clarity, safety, correctness, or stronger evidence.
- Ask them to predict one realistic follow-up bug, objection, misunderstanding, or failure mode and where they would look first.
- Use the closing probe from the workflow and feed its answer into `move_down_plan`.

For `0/1/2`, generative probes test authorship consistency, not perfect present recall. A rusty author may forget details and still show author-consistent familiarity: where the work came from, why it is structured that way, what alternatives were considered, what was hard, and what they would inspect first to recover context.

## Question Rules

Before asking each scoring question, run this self-test:

- Can the answer be recovered from the question text alone?
- Did the question include a diff snippet, implementation summary, expected answer, or likely failure mode?
- Is it yes/no when explanation is needed?
- Does it ask multiple questions?

If any answer is yes, rewrite.

Allowed context: locate the area by function, file, command, test name, artifact section, thread, or feature. State the requested answer type: invariant, failure mode, alternative, test gap, or evidence gap.

Banned context: do not paste the diff, explain behavior, state why an invariant exists, reveal the likely failure mode, or ask a question where the desired answer is obvious from wording.

Ask cold first. "Stuck" means no substantive answer, generic answer, asking you to explain first, confident wrong answer, repeated "not sure" on material behavior, refusal, or persistently thin answers. Give at most one hint after stuck. A hint may locate file, function, or feature, but must not explain behavior or reveal the answer.

## Grading

Classify each answer immediately, but do not over-explain during the interview.

- Strong evidence: cites specific behavior not supplied by you; names relevant functions, paths, tests, flags, schemas, claims, data, thread context, or runtime conditions; predicts plausible failure modes; explains evidence limits; gives a rejected alternative or small extension.
- Weak evidence: repeats title, broad intent, or your wording; uses generic implementation language; cannot connect tests, evidence, or claims to risk; explains only high-level outcome; agrees without independent evidence.
- Failed evidence: no substantive answer; generic answer after a targeted probe; asks you to explain first; confident wrong answer; repeated "not sure" on material behavior; refusal or persistently thin answers.

Honest uncertainty is better than confident wrong, but uncertainty is not proof of ownership. If provenance is weak or agentic, refusal or thin answers keep the result at `5/5` with high confidence unless the artifact itself proves a narrower claim.

## Scoring

Use ledger-first scoring, not gestalt:

1. Write a probe ledger with counts by probe type and grade: provenance, comprehension, generative, each graded `strong`, `partial`, `weak`, or `failed`.
2. Derive the integer band from provenance and probe evidence without using self-score.
3. Derive the decimal from severity inside that band without using self-score.
4. Compare the provisional agent score to self-score only after steps 1-3.

Band gates:

- `0/5`: Credible no-agent attestation plus generative evidence that verifies human authorship.
- `1/5`: Credible auxiliary-only agent attestation plus generative evidence that the author owns the substance.
- `2/5`: Agent wrote substantial content under direct guidance, and evidence shows the author shaped design, constraints, or final content enough to own direction.
- `3/5`: Agent produced most substance, and evidence shows detailed review ownership across material areas.
- `4/5`: Author has high-level understanding plus real sanity checks, but misses important details or cannot own extensions.
- `5/5`: Broad delegation, no meaningful review, refusal, thin answers, major gaps in material behavior, or zero demonstrated understanding.

A failed material provenance or generative probe blocks `0/1/2` unless other direct evidence strongly verifies the claim. For `3/5`, require detailed review ownership. If detail probes are mostly weak or failed, use `4/5` or higher.

Decimal rules:

- Strong-majority evidence lowers the decimal inside a band.
- Weak or failed material probes raise it inside the band.
- If evidence only supports a band, use `.0` and lower confidence instead of inventing precision.

Exact-match tripwire: if `abs(agent_score - human_self_score) < 0.3`, rerun the derivation from the probe ledger with self-score ignored. Keep the close match only if the ledger independently supports it. Mention the tripwire result in `disagreement` or `understanding_basis`.

If the author raises a correct flaw in the rubric or audit method, treat that as instrument feedback. Do not lower the slop score for noticing the flaw unless the answer also proves artifact ownership.

## Output

Return the final verdict as one short paragraph, at most two sentences. Do not use default bullets, fenced code blocks, Markdown tables, or internal labels like `ledger`.

Include agent score, self-score if known, confidence, and the strongest basis. Mention failed probes or move-down work only when they materially affect the score. If the user asks for full detail, then expand into bullets with `probe_ledger`, `provenance_basis`, `understanding_basis`, `failed_probes`, `disagreement`, and `move_down_plan`.

Treat the paragraph as author-owned self-disclosure for the audited artifact. Do not attach audit output to another person's contribution as if it were an external judgment.

Do not soften the score to match self-score. The author may contest with concrete evidence, but the agent owns the advisory verdict.

## Example

Artifact: benchmark report claiming a scheduler prefetch change improves p95 latency by 11% on workload A.

Author self-score: `3/5`

Provenance claim: "Agent drafted the report and plots from my commands. I designed the experiment, ran the benchmark, reviewed the numbers, and rewrote the caveats."

Probe transcript:

1. Confounder probe: author names run order, cache or allocator warmup, randomized A/B order, and dropped warmup iterations. Grade `strong`.
2. Evidence-limit probe: author says p95 chart proves latency distribution at fixed load, not throughput. Grade `strong`.
3. Rejected-framing probe: author rejected "prefetcher is faster" because workload B was neutral and sample was narrow. Grade `strong`.
4. Closing probe: author would add a reproducibility appendix with commands, seed policy, and environment hash. Grade `strong`.

Verdict:

Self-assessed benchmark report is `2.4/5` slop with high confidence; the author self-scored `3/5`. Agent drafted substantial content, but the author owned experiment design, benchmark execution, number review, caveats, and evidence limits.
