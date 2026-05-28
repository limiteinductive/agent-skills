---
name: slope-audit
description: Assess how agentic a PR, code change, document, or technical contribution is on a 0/5 to 5/5 slope scale. Use when Codex needs to interview an author about PR ownership, human understanding, agent involvement, review depth, production readiness transparency, or when the user invokes $slope-audit.
---

# Slope Audit

Evaluate author ownership of a contribution by combining provenance claims with an adversarial but fair interview. The audit is advisory. Do not block production work. Assign the final score yourself and show the author's self-score separately.

## Scale

Use this scale exactly:

- `0/5`: No agent involvement. Everything was written manually by a human.
- `1/5`: Agent used only for mechanical or auxiliary tasks: running git commands, writing commit messages, formatting, or reviewing.
- `2/5`: Agent wrote a substantial part of the content under direct human guidance. Human could have written it themselves. Agent use was mostly convenience.
- `3/5`: Agent produced most ideas or implementation. Human closely reviewed output and fully understands it.
- `4/5`: Agent produced most ideas or implementation. Human understands it at a high level and verified sanity through tests, benchmarks, or checks.
- `5/5`: Human gave broad request, agent did it, and human did not meaningfully read or review the output.

Treat `slope` as the user-chosen name, despite the origin being AI slop. Do not call this the Benji scale.

## Workflow

1. Inspect the artifact before interviewing. For PRs, inspect the diff, commits, tests, checks, touched code, and stated purpose. Prefer real files and commands over summaries.
2. Ask the author for two things first: their self-score and a brief agent-use claim. Treat this as a claim to test, not as truth.
3. Start the working score at `5/5`. Lower it only when provenance plus interview evidence supports a lower score.
4. Ask freeform technical questions in normal chat, one question at a time. Do not use multiple-choice questions for scoring probes.
5. Run at least 4 scoring probes unless the author refuses to engage. Cap the interview at 8 scoring probes plus 2 follow-ups.
6. Stop when the likely score bracket is within 1 point at medium or high confidence, two consecutive scoring probes do not change the bracket, or the cap is reached.
7. Produce the output contract. Include disagreement between author self-score and agent score.

If the author has the PR open during the interview, state that the audit measures current review and maintenance ownership, not necessarily what they knew while producing the change.

## Probe Types

Use a mix of these probes. Pick questions from the actual diff and risk surface.

### Provenance Probes

Use these to separate `0/1/2` from `3/4/5`.

- Ask who produced the idea, structure, implementation, tests, review comments, and final text.
- Ask what the author wrote manually, what the agent generated, and what they changed after agent output.
- Ask which parts they would be comfortable owning without the agent present.
- Require attestation for `0/1/2`. Comprehension alone cannot prove these levels.

### Comprehension Probes

Use these to separate `3/4/5`.

- Ask about invariants, data flow, control flow, failure modes, test meaning, rollout risk, and backward compatibility.
- Ask what would break if one condition, flag, helper, schema field, or ordering changed.
- Ask why a test proves the intended behavior and what it does not prove.
- Ask how runtime behavior differs before and after the PR.

### Generative Probes

Use these to separate `0/1/2/3`.

- Ask how the author would implement a nearby extension live.
- Ask what alternative implementation they rejected and why.
- Ask how they would rewrite a small area for clarity or safety.
- Ask them to predict one realistic follow-up bug and where they would look first.

Generative ownership is the main evidence for "I could have written this myself." Passive understanding is not enough.

## Question Rules

Before asking each scoring question, run this self-test:

- Can the answer be recovered from the question text alone?
- Did the question include a diff snippet, implementation summary, expected answer, or likely failure mode?
- Is it yes/no when explanation is needed?
- Does it ask multiple questions?

If any answer is yes, rewrite the question.

Allowed context:

- Locate the area: function, file, command, test name, PR section, or feature.
- State what kind of answer is expected: invariant, failure mode, alternative, test gap.

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
- Names relevant functions, paths, tests, flags, schemas, or runtime conditions.
- Predicts plausible failure modes.
- Explains test coverage limits.
- Gives a plausible rejected alternative.
- Performs a small design extension without relying on your explanation.

Weak evidence:

- Repeats the PR title or broad intent.
- Gives generic implementation language.
- Restates your wording.
- Cannot connect tests to risk.
- Explains only high-level outcome.
- Agrees with your framing without adding independent evidence.

Failure evidence:

- No substantive answer.
- Generic answer after a targeted probe.
- Asking you to explain the relevant code first.
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
- `5/5`: Broad delegation, no meaningful review, refusal, thin answers, or major gaps in material behavior.

Do not lower below `3/5` from comprehension probes alone. Use provenance plus generative ownership for `0/1/2`.

## Output

Return this structure:

```text
human_self_score: <score or unknown>
agent_score: <0/5 to 5/5>
confidence: <high | medium | low>
provenance_basis: <short evidence>
understanding_basis: <short evidence>
failed_probes: <none or concise list>
disagreement: <none or self-score vs agent-score gap>
move_down_plan: <specific work to reduce score>
public_label: <one sentence suitable for PR or review notes>
```

Keep the final verdict terse and evidence-based. Do not soften the score to match the author's self-score. The author may contest with concrete evidence, but the agent owns the advisory verdict.
