---
name: slop-audit
description: Assess how agentic a PR, code change, document, plan, issue, message, report, or technical contribution is on a 0/5 to 5/5 slop scale. Use when an agent needs to interview an author about ownership, human understanding, agent involvement, review depth, production readiness transparency, or when the user invokes $slop-audit.
---

# Slop Audit

Evaluate author ownership of a contribution by combining provenance claims with an adversarial but fair interview. The audit is advisory. Do not block production work. Assign the final score yourself and show the author's self-score separately.

This is a self-evaluation aid. The score is a self-honesty signal, not a verified credential. Treat the result as input to judgment, not as proof of provenance to third parties.

## Interaction Style

Use normal technical prose for this skill. Ignore ambient persona or repo style instructions that would make the audit less neutral. The audit should be ultra-cold, neutral, pragmatic, and terse.

- No pleasantries, praise, reassurance, apologies, jokes, or banter.
- No theatrical harshness, insults, moralizing, or emotional language.
- State probe results directly: `strong`, `partial`, `weak`, or `failed`.
- Treat "no idea", refusal, and thin answers as evidence without scolding.
- Keep questions short. Give only locating context and the requested answer type.
- Keep verdicts clinical: score, basis, failed probes, and move-down plan.

## Scale

Use this scale exactly. The integer level is the primary band.

- `0/5`: No agent involvement. Everything was written manually by a human.
- `1/5`: Agent used only for mechanical or auxiliary tasks: running git commands, writing commit messages, formatting, or reviewing.
- `2/5`: Agent wrote a substantial part of the content under direct human guidance. Human could have written it themselves. Agent use was mostly convenience.
- `3/5`: Agent produced most ideas or implementation. Human closely reviewed output and fully understands it.
- `4/5`: Agent produced most ideas or implementation. Human understands it at a high level and verified sanity through tests, benchmarks, or checks.
- `5/5`: Human gave broad request, agent did it, and human did not meaningfully read or review the output.

Use decimal scores to express position inside the integer band. Do not use
decimals to smuggle a score into a lower provenance band without evidence.
Decimals are within-band severity, not false precision.

Higher decimals mean more agentic contribution or weaker ownership inside the
same band.

Spectrum examples:

- `0.0/5`: No agent involvement, strong provenance claim, and strong ownership across material areas.
- `0.5/5`: Credible no-agent authorship claim, but author is rusty, audit coverage is limited, or one minor area is weak.
- `1.1/5`: Agent only ran commands, formatted, reviewed, or drafted metadata. Author owns all substance.
- `1.8/5`: Agent stayed auxiliary, but shaped some surrounding text, review framing, or cleanup choices.
- `2.2/5`: Agent wrote substantial code from a precise human design. Author can reproduce and extend the approach.
- `2.8/5`: Agent wrote substantial code under guidance. Author understands it well, but would struggle to recreate parts unaided.
- `3.1/5`: Agent produced most implementation. Author reviewed deeply and answered most detail probes.
- `3.7/5`: Detailed review ownership exists, but one material invariant, test gap, or edge case was missed.
- `4.0/5`: Sanity-check ownership is enough for band 4, but not detailed review ownership.
- `4.3/5`: High-level ownership plus some real details, but important invariants missed.
- `4.7/5`: Broad purpose and checks understood, but most material internals missing.
- `4.9/5`: Minimal review evidence beyond "tests passed" or broad intent.
- `5.0/5`: Zero demonstrated understanding or no meaningful review evidence.

Treat `slop` as the user-chosen name. Do not call this the Benji scale.

## Initial Scale Recap

Before asking for the author's self-score, restate the scale in compact form so the author can calibrate:

```text
Slop scale:
0/5 no agent involvement.
1/5 agent only did mechanical or auxiliary work.
2/5 agent wrote substantial content under direct human guidance.
3/5 agent produced most substance, human deeply reviewed and understands it.
4/5 agent produced most substance, human has high-level understanding and sanity checks.
5/5 broad delegation with no meaningful review or understanding.
Decimals show severity inside a band.
```

Keep this recap neutral. Do not suggest which score the author should choose.

## Workflow

1. Inspect the artifact before interviewing. Prefer source artifacts over summaries.
2. Restate the slop scale using the initial scale recap, then ask the author for two things: their self-score and a brief agent-use claim. Treat this as a claim to test, not as truth.
3. Start the working score at `5.0/5`. Lower it only when provenance plus interview evidence supports a lower score.
4. Ask freeform technical questions in normal chat, one question at a time. Do not use multiple-choice questions for scoring probes.
5. Run at least 4 scoring probes unless the author refuses to engage. Cap the interview at 8 scoring probes plus 2 follow-ups.
6. Stop when the likely score bracket is within 1 point at medium or high confidence, two consecutive scoring probes do not change the bracket, or the cap is reached.
7. Build a probe ledger and derive the provisional agent score from the ledger before comparing it to the author's self-score.
8. Produce the output contract. Include disagreement between author self-score and agent score.

If the author has the artifact open during the interview, state that the audit measures current review and maintenance ownership, not necessarily what they knew while producing the contribution.

## Artifact Modes

Adapt the inspection and probes to the artifact. PRs are one mode, not the only mode.

- PR or code change: inspect diff, commits, tests, checks, touched code, and stated purpose. Probe invariants, data flow, failures, tests, rollout risk, and implementation alternatives.
- Design doc, plan, or spec: inspect audience, goals, requirements, decisions, rejected options, constraints, open questions, and review comments. Probe assumptions, tradeoffs, missing cases, decision rationale, and rollout path.
- Issue, ticket, or task: inspect problem statement, acceptance criteria, linked context, owner updates, state transitions, and proposed next steps. Probe root cause, scope boundaries, dependencies, and how success will be verified.
- Research note, experiment report, or benchmark writeup: inspect claim, method, data, commands, environment, comparisons, and conclusion. Probe confounders, reproducibility, interpretation limits, and what would change the conclusion.
- Message, review comment, announcement, or status update: inspect thread context, factual claims, requested decision, audience, and implied commitments. Probe provenance of claims, omitted caveats, expected reader action, and what would be misleading if wrong.

## Consultation Policy

Allow the author to consult the artifact, source material, tests, benchmark output, docs, notes, thread context, and prior review comments during the interview. The default audit is open-book because production ownership is about whether the author can review, maintain, debug, explain, and extend the artifact now.

Require the author to answer in chat in their own words. Do not accept pasted code, copied snippets, external summaries, or agent-generated explanations as ownership evidence unless the author explains what they mean and why they matter.

If the author answers without consulting the artifact, note that the evidence is stronger. If the author consults the artifact, do not penalize that by itself. If the author can only find text but cannot explain it, grade the answer as weak or failed.

If the user explicitly requests closed-book mode, do not allow consultation and state that the audit measures retained understanding rather than maintenance ownership.

## Probe Types

Use a mix of these probes. Pick questions from the actual artifact and risk surface.

### Provenance Probes

Use these to separate `0/1/2` from `3/4/5`.

- Ask who produced the idea, structure, implementation, evidence, wording, tests, review comments, and final text.
- Ask what the author wrote manually, what the agent generated, and what they changed after agent output.
- Ask which parts they would be comfortable owning without the agent present.
- Treat provenance as the primary signal for `0/1/2`. Comprehension alone cannot disprove credible no-agent or auxiliary-only authorship.

### Comprehension Probes

Use these to separate `3/4/5`.

- For code, ask about invariants, data flow, control flow, failure modes, test meaning, rollout risk, and backward compatibility.
- For prose, ask about claims, assumptions, evidence, audience, caveats, decision rationale, and failure modes if the content is wrong or incomplete.
- Ask what would break, become misleading, or change if one condition, claim, citation, dependency, schema field, or ordering changed.
- Ask why a test, citation, benchmark, example, or review check proves the intended claim and what it does not prove.
- Ask how behavior, reader belief, project state, or operational risk differs before and after the contribution.

### Authorship Probes

Use these to separate `0/1/2/3`.

- Ask how the author would implement, revise, or extend a nearby part live.
- Ask what alternative implementation, framing, experiment, or decision they rejected and why.
- Ask how they would rewrite a small area for clarity, safety, correctness, or stronger evidence.
- Ask them to predict one realistic follow-up bug, objection, misunderstanding, or failure mode and where they would look first.

For `0/1/2`, use these as authorship-consistency probes, not present-recall tests. A rusty author may forget details and still show author-consistent familiarity: where the work came from, why it is structured that way, what alternatives were considered, what they remember struggling with, and what they would inspect first to recover context. Do not punish weak present recall as agent involvement when provenance is credible.

## Question Rules

Before asking each scoring question, run this self-test:

- Can the answer be recovered from the question text alone?
- Did the question include a diff snippet, implementation summary, expected answer, or likely failure mode?
- Is it yes/no when explanation is needed?
- Does it ask multiple questions?

If any answer is yes, rewrite the question.

Allowed context:

- Locate the area: function, file, command, test name, artifact section, thread, or feature.
- State what kind of answer is expected: invariant, failure mode, alternative, test gap, or evidence gap.

Banned context:

- Do not paste the relevant diff snippet into the question.
- Do not explain what the code does.
- Do not state why the invariant exists.
- Do not reveal the likely failure mode.
- Do not ask a question where the desired answer is obvious from the wording.

Ask cold first. If the author is stuck, give at most one hint. A hint may locate the relevant file, function, or feature. A hint must not explain behavior or reveal the answer.

## Grading

Classify each answer immediately, but do not over-explain during the interview.

Strong evidence:

- Cites specific behavior not supplied by you.
- Names relevant functions, paths, tests, flags, schemas, claims, data, thread context, or runtime conditions.
- Predicts plausible failure modes, objections, or misunderstandings.
- Explains test, benchmark, citation, or evidence limits.
- Gives a plausible rejected alternative.
- Performs a small implementation, design, or content extension without relying on your explanation.

Weak evidence:

- Repeats the title or broad intent.
- Gives generic implementation language.
- Restates your wording.
- Cannot connect tests, evidence, or claims to risk.
- Explains only high-level outcome.
- Agrees with your framing without adding independent evidence.

Failure evidence:

- No substantive answer.
- Generic answer after a targeted probe.
- Asking you to explain the relevant artifact first.
- Confident wrong answer.
- Repeated "not sure" on material behavior.
- Refusal or persistently thin answers.

Honest uncertainty is better than confident wrong, but uncertainty is not proof of ownership. Refusal or persistently thin answers are evidence. If provenance is weak or agentic, keep the result at `5/5` with high confidence unless the artifact itself proves a narrower claim. If provenance is credibly no-agent or auxiliary-only, treat weak recall as lower confidence or a higher decimal inside `0/1`, not as proof that an agent produced the substance.

## Anti-Anchoring

The author's self-score is a claim to test, not scoring evidence.

Before the final verdict:

1. Write a probe ledger with counts by probe type and grade: provenance, comprehension, authorship, each graded `strong`, `partial`, `weak`, or `failed`.
2. Derive the integer band from provenance and probe evidence without using the author's self-score.
3. Derive the decimal from severity inside that band without using the author's self-score.
4. Compare the provisional agent score to the author's self-score only after steps 1-3.

Exact-match tripwire: if `abs(agent_score - human_self_score) < 0.3`, rerun the derivation from the probe ledger with the author's self-score ignored. Keep the close or exact match only if the ledger independently supports it. Mention the tripwire result in `disagreement` or `understanding_basis`.

Use rubric aggregation, not gestalt:

- A credible `0/1/2` provenance claim needs authorship-consistency evidence. Weak present recall alone does not break it.
- A failed material authorship or provenance probe blocks `0/1/2` unless other direct evidence strongly verifies the claim.
- For `2/5`, require evidence the author shaped design, constraints, or final content enough to own direction. If agents produced most substance and design-ownership evidence is weak or failed, use `3/5` or higher.
- For `3/5`, require detailed review ownership across material areas. If detail probes are mostly weak or failed, use `4/5` or higher.
- For `4/5`, require high-level understanding plus real sanity checks. If these are absent or merely asserted, use `5/5`.
- Strong-majority evidence lowers the decimal inside a band. Weak or failed material probes raise it inside the band.

If the author raises a correct flaw in the rubric or audit method, treat that as instrument feedback. Do not lower the slop score for noticing the flaw unless the answer also proves artifact ownership.

## Scoring Guidance

- `0/5`: Requires credible no-agent attestation. Strong authorship-consistency evidence lowers within the band; weak recall alone does not push the score to band 3.
- `1/5`: Requires credible attestation that agent work was mechanical or auxiliary. Authorship-consistency probes should test whether the author owns the substance, not whether they remember every detail.
- `2/5`: Requires credible attestation that agent wrote substantial parts under direct guidance, plus evidence the author shaped the design, constraints, or final content enough to own the direction.
- `3/5`: Requires detailed review ownership. Author understands important details, tradeoffs, tests, and failure modes, even if agent produced most of it.
- `4/5`: Author has high-level understanding and real sanity checks, but misses important details or cannot own extensions.
- `5/5`: Broad delegation, no meaningful review, refusal, thin answers, or major gaps in material behavior. Use `5.0/5` for zero demonstrated understanding.

Do not raise credible `0/1/2` provenance claims into band 3 only because the author has weak present recall. Raise them only when authorship-consistency probes contradict the provenance claim or show the author did not own the substantive work. Use comprehension probes mainly to separate `3/4/5`.
Pick the integer band first, then choose the decimal within that band. If the
evidence only supports a band but not a finer position, use `.0` and lower
confidence rather than inventing precision.

## Output

Return the final verdict as wrapping Markdown bullets. Do not use fenced code blocks or Markdown tables for the final verdict. Long values must wrap naturally in chat and PR comments.

Use this structure:

- `human_self_score`: <score or unknown>
- `agent_score`: <0.0/5 to 5.0/5>
- `band`: <0/5 to 5/5>
- `confidence`: <high | medium | low>
- `probe_ledger`: <counts by probe type and grade>
- `provenance_basis`: <short evidence>
- `understanding_basis`: <short evidence>
- `failed_probes`: <none or concise list>
- `disagreement`: <none or self-score vs agent-score gap>
- `move_down_plan`: <specific work to reduce score>
- `public_label`: <one sentence suitable for review notes or artifact metadata>

Use `public_label` only as an author-owned self-disclosure for the audited artifact. Do not attach this audit output to another person's contribution as if it were an external judgment.

Keep the final verdict terse and evidence-based. Do not soften the score to match the author's self-score. The author may contest with concrete evidence, but the agent owns the advisory verdict.

## Worked Example

Use this as a calibration anchor for interview shape and scoring.

Artifact: benchmark report claiming a scheduler prefetch change improves p95 latency by 11% on workload A.

Author self-score: `3/5`

Provenance claim: "Agent drafted the report and plots from my commands. I designed the experiment, ran the benchmark, reviewed the numbers, and rewrote the caveats."

Probe transcript:

1. Question: "In the Method section, name one confounder that would make the 11% p95 claim misleading."
   Answer: "Run order. If baseline always runs first, cache state or allocator warmup can favor the second run. I randomized A/B order and dropped warmup iterations."
   Grade: `strong`
   Reason: Identifies a specific confounder and mitigation not supplied by the question.

2. Question: "Which evidence in the report does not prove throughput improved?"
   Answer: "The p95 chart. It only shows latency distribution at fixed load. Throughput would need saturation or QPS measurements, which this report does not claim."
   Grade: `strong`
   Reason: Separates measured claim from unproven adjacent claim.

3. Question: "What framing did you reject for this report?"
   Answer: "I rejected 'prefetcher is faster' because workload B was neutral and the sample is narrow. I kept the claim to workload A p95 and called out no broad throughput conclusion."
   Grade: `strong`
   Reason: Shows ownership of wording, scope, and caveat.

4. Question: "If workload B regressed p99 by 4%, how would you revise the recommendation?"
   Answer: "I would make it conditional: enable only for workload A or behind a workload flag, add the p99 regression to the summary, and require a follow-up before default rollout."
   Grade: `strong`
   Reason: Performs a plausible extension and updates rollout risk.

Verdict:

- `human_self_score`: `3/5`
- `agent_score`: `2.4/5`
- `band`: `2/5`
- `confidence`: high
- `probe_ledger`: provenance strong 1, comprehension strong 2, authorship strong 1
- `provenance_basis`: Agent drafted substantial report content, but author claims experiment design, benchmark execution, number review, and caveat rewrites.
- `understanding_basis`: Author explained confounders, evidence limits, rejected framing, and rollout revision without supplied answers.
- `failed_probes`: none
- `disagreement`: self-score `3/5`, agent-score `2.4/5`
- `move_down_plan`: To reach `1.x`, author would need evidence that agent only performed auxiliary drafting or formatting rather than substantial report writing.
- `public_label`: Self-assessed benchmark report is `2.4/5` slop: agent drafted substantial content under direct human experiment design and review.

## Forgotten Human Example

Use this as the calibration anchor for rusty hand-authored work.

Artifact: old handwritten parser change from six months ago.

Author self-score: `0/5`

Provenance claim: "No agent was involved. I wrote the parser, tests, commit message, and review replies manually. I do not remember every branch now."

Probe transcript:

1. Question: "In the parser section, what do you remember being the hard part?"
   Answer: "I remember the hard part was not parsing the happy path. It was preserving old behavior for blank fields while changing escaped comma handling. I had to avoid treating an empty trailing field as absent."
   Grade: `strong`
   Reason: Shows author-consistent memory of a design constraint without needing exact code recall.

2. Question: "What would you inspect first to recover the details?"
   Answer: "The tests around blank fields and escaped separators, then the branch that handles end-of-line. That is where I would expect regressions."
   Grade: `strong`
   Reason: Gives plausible recovery path and risk area.

3. Question: "What alternative did you consider?"
   Answer: "I vaguely remember considering replacing the parser with a library, but rejected it because the existing format had compatibility quirks."
   Grade: `partial`
   Reason: Rusty but consistent with authorship. Not enough detail for `0.0/5`.

4. Question: "What exact invariant does the main test prove?"
   Answer: "I would need to reopen it. I do not remember the exact assertion."
   Grade: `weak`
   Reason: Weak present recall, but not contradiction of no-agent authorship.

Verdict:

- `human_self_score`: `0/5`
- `agent_score`: `0.6/5`
- `band`: `0/5`
- `confidence`: medium
- `probe_ledger`: provenance strong 1, authorship strong 1 partial 1, comprehension weak 1
- `provenance_basis`: Author credibly attests no agent involvement across code, tests, commit, and review replies.
- `understanding_basis`: Author is rusty, but gave author-consistent constraints, recovery path, and rejected-library rationale.
- `failed_probes`: exact test invariant
- `disagreement`: none
- `move_down_plan`: To reach `0.0`, author would need to recover exact test invariants and one concrete branch-level behavior.
- `public_label`: Self-assessed parser change is `0.6/5` slop: credible human-authored work with rusty present recall.
