---
name: slope-audit
description: Assess how agentic a PR, code change, document, plan, issue, message, report, or technical contribution is on a 0/5 to 5/5 slope scale. Use when Codex needs to interview an author about ownership, human understanding, agent involvement, review depth, production readiness transparency, or when the user invokes $slope-audit.
---

# Slope Audit

Evaluate author ownership of a contribution by combining provenance claims with an adversarial but fair interview. The audit is advisory. Do not block production work. Assign the final score yourself and show the author's self-score separately.

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
- `0.5/5`: No agent involvement claimed and mostly supported, but audit coverage is limited or one minor area is weak.
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

Treat `slope` as the user-chosen name, despite the origin being AI slop. Do not call this the Benji scale.

## Workflow

1. Inspect the artifact before interviewing. Prefer source artifacts over summaries.
2. Ask the author for two things first: their self-score and a brief agent-use claim. Treat this as a claim to test, not as truth.
3. Start the working score at `5.0/5`. Lower it only when provenance plus interview evidence supports a lower score.
4. Ask freeform technical questions in normal chat, one question at a time. Do not use multiple-choice questions for scoring probes.
5. Run at least 4 scoring probes unless the author refuses to engage. Cap the interview at 8 scoring probes plus 2 follow-ups.
6. Stop when the likely score bracket is within 1 point at medium or high confidence, two consecutive scoring probes do not change the bracket, or the cap is reached.
7. Produce the output contract. Include disagreement between author self-score and agent score.

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
- Require attestation for `0/1/2`. Comprehension alone cannot prove these levels.

### Comprehension Probes

Use these to separate `3/4/5`.

- For code, ask about invariants, data flow, control flow, failure modes, test meaning, rollout risk, and backward compatibility.
- For prose, ask about claims, assumptions, evidence, audience, caveats, decision rationale, and failure modes if the content is wrong or incomplete.
- Ask what would break, become misleading, or change if one condition, claim, citation, dependency, schema field, or ordering changed.
- Ask why a test, citation, benchmark, example, or review check proves the intended claim and what it does not prove.
- Ask how behavior, reader belief, project state, or operational risk differs before and after the contribution.

### Generative Probes

Use these to separate `0/1/2/3`.

- Ask how the author would implement, revise, or extend a nearby part live.
- Ask what alternative implementation, framing, experiment, or decision they rejected and why.
- Ask how they would rewrite a small area for clarity, safety, correctness, or stronger evidence.
- Ask them to predict one realistic follow-up bug, objection, misunderstanding, or failure mode and where they would look first.

Generative ownership is the main evidence for "I could have written this myself." Passive understanding is not enough.

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

Honest uncertainty is better than confident wrong, but uncertainty is not proof of ownership. Refusal or persistently thin answers are evidence. In that case, keep the result at `5/5` with high confidence unless the artifact itself proves a narrower claim.

## Scoring Guidance

- `0/5`: Requires explicit no-agent attestation and strong generative ownership across material areas.
- `1/5`: Requires attestation that agent work was mechanical or auxiliary, plus strong generative ownership.
- `2/5`: Requires attestation that agent wrote substantial parts under direct guidance, plus evidence the author could reproduce or extend the approach.
- `3/5`: Requires detailed review ownership. Author understands important details, tradeoffs, tests, and failure modes, even if agent produced most of it.
- `4/5`: Author has high-level understanding and real sanity checks, but misses important details or cannot own extensions.
- `5/5`: Broad delegation, no meaningful review, refusal, thin answers, or major gaps in material behavior. Use `5.0/5` for zero demonstrated understanding.

Do not lower below `3/5` from comprehension probes alone. Use provenance plus generative ownership for `0/1/2`.
Pick the integer band first, then choose the decimal within that band. If the
evidence only supports a band but not a finer position, use `.0` and lower
confidence rather than inventing precision.

## Output

Return this structure:

```text
human_self_score: <score or unknown>
agent_score: <0.0/5 to 5.0/5>
band: <0/5 to 5/5>
confidence: <high | medium | low>
provenance_basis: <short evidence>
understanding_basis: <short evidence>
failed_probes: <none or concise list>
disagreement: <none or self-score vs agent-score gap>
move_down_plan: <specific work to reduce score>
public_label: <one sentence suitable for review notes or artifact metadata>
```

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

```text
human_self_score: 3/5
agent_score: 2.4/5
band: 2/5
confidence: high
provenance_basis: Agent drafted substantial report content, but author claims experiment design, benchmark execution, number review, and caveat rewrites.
understanding_basis: Author explained confounders, evidence limits, rejected framing, and rollout revision without supplied answers.
failed_probes: none
disagreement: self-score 3/5, agent-score 2.4/5
move_down_plan: To reach 1.x, author would need evidence that agent only performed auxiliary drafting or formatting rather than substantial report writing.
public_label: Benchmark report is 2.4/5 slope: agent drafted substantial content under direct human experiment design and review.
```
