---
name: converge
description: Adversarial convergence loop using critic + codex. Modes — review (specs, plans, docs), diagnose (bugs, root cause), implement (code a spec step-by-step with review at each step). Trigger on "converge", "/converge".
---

# Converge — Constructive Adversarial Convergence

Two independent reviewers — different model families, different failure modes — review the same target in parallel, then cross-examine each other's findings. Like peer review in science: rigorous, evidence-based, but aimed at making the work better, not winning an argument.

Based on multi-agent debate research (Du et al. 2023, Liang et al. 2023, Anthropic debate alignment work): independent parallel review followed by structured cross-critique outperforms sequential review. 2-3 rounds is optimal. Model diversity (Claude + GPT) is itself a strong debiasing technique.

## Why two reviewers from different families

- **Critic (Claude)** — catches logical gaps, hidden assumptions, structural issues, second-order effects
- **Codex (GPT-5.4)** — catches implementation bugs, code mismatches, wrong line numbers, missing edge cases
- **Different training data and failure modes** — what one model misses, the other often catches (Chan et al. 2023: role-diverse panels outperform homogeneous ones)
- **Independent-then-compare** — both review the target blind (no prior findings), then cross-critique in subsequent rounds

## Arguments

```
/converge <target-or-description> [--mode review|diagnose|implement] [--rounds N] [--min-rounds N] [--focus "question"] [--severity high|medium|low] [--pre-scan "angle"]
```

- `<target-or-description>` — file path, bug description, or free-text context
- `--mode` — auto-detected from context if not specified (see below)
- `--rounds N` — max rounds before stopping (default: 3). `--min-rounds` always enforces at least 2 regardless.
- `--min-rounds N` — minimum rounds before convergence is allowed (default: 2). Cannot be set below 2.
- `--focus "question"` — optional focus area to guide reviewers
- `--severity high|medium|low` — convergence threshold (default: medium). Converge only when no new findings at or above this severity remain. Low-severity nits do not block convergence.
- `--pre-scan "angle"` — run a focused preliminary scan before round 1 (see "Pre-scan" below). Example angles: "security", "performance", "user-facing behavior", "edge cases".

### Mode auto-detection

If no `--mode` is given, infer from context:
- **File path to a doc** (spec, plan, design, `.md`) → `review` (unless the file content describes a bug — then `diagnose`)
- **Bug description, error message, "why does X happen"** → `diagnose`
- **"implement", "build", "code", references to a spec + impl plan** → `implement`
- **Source code file** (`.ts`, `.py`, etc.) → `review`
- **Ambiguous** → ask the user. After clarification, restart from the beginning with the chosen mode.

### Input validation

Before launching any review:
- **File target:** verify the file exists. If not, report the error and stop.
- **Bug description without reproduction steps:** ask the user for steps before proceeding. Block until provided.
- **Empty or missing target:** report the error and stop.

### Pre-scan

When `--pre-scan "angle"` is provided, run a single focused Codex pass **before** round 1 begins. The pre-scan reviews the target through a specific lens (security, performance, user-facing behavior, etc.) and produces a list of angle-specific findings.

**Why pre-scan instead of a third reviewer:** Same-model instances (two GPT-5.4s) have highly correlated failure modes — prompt diversity is weaker than model diversity for catching different bugs. A third reviewer from the same family adds compute without proportional coverage gain. Pre-scan is cheaper (one extra call, not 50% more per round) and its findings get properly cross-examined.

**How it works:**
1. Launch one Codex call with the angle prompt: "Review this target specifically for [angle]. Return findings with evidence."
2. Collect pre-scan findings and label them `PS1`, `PS2`, etc.
3. Feed pre-scan findings as **additional context** (not as a reviewer's round 1 output) to both Critic and Codex in round 1: "A preliminary scan flagged these items. Verify, dispute, or confirm each as part of your review."
4. Both real reviewers can cross-examine pre-scan findings in the normal 2-reviewer flow — no orphaned or unverified findings.

The pre-scan does NOT count as a round. It is a context-enrichment step.

**Angle selection:** If `--pre-scan` is provided without a specific angle, infer a sensible default from the target type:
- Source code → "security and error handling"
- API spec / schema → "breaking changes and backwards compatibility"
- Design doc / plan → "feasibility and missing requirements"
- General / unclear → ask the user for the angle

---

## Round structure (all modes)

All modes use the same round structure. Mode-specific differences are in what gets reviewed, not how rounds work.

**Round 1 — Independent review (blind):**
1. Launch both reviewers in parallel. Neither sees the other's output. Each reviews independently. If `--pre-scan` was used, both reviewers receive the pre-scan findings as additional context to verify.
2. Collect findings — each returns claims with evidence, classified by severity (High/Medium/Low).
3. Synthesize into convergence table. Apply agreed fixes immediately.

**Round 2 — Cross-critique (always runs):**
1. Send each reviewer the OTHER's round 1 findings with the debiasing prompt: "Assume the other review contains at least one error. Identify it with evidence, or explain with evidence why each finding is correct."
2. Each reviewer: confirms, disputes with counter-evidence, or adds findings they missed in round 1.
3. Synthesize. Apply agreed fixes. Flag disagreements.
4. **After round 2:** output the progress line and check user input (see "User controls"). If no input, check convergence criteria. If not converged AND max rounds > 2, continue to round 3.

**Round 3+ — Focused resolution (only if unresolved High/Medium findings remain):**
1. Only send unresolved findings — NOT the full target. Scope the prompt: "These N findings remain in dispute. For each, provide your final position with evidence."
2. Check for cycling before synthesizing (see below).
3. If converged or max rounds reached, stop and produce report.

### Convergence criteria (unified, all modes)

Stop when ALL of:
- At least `--min-rounds` rounds completed (minimum 2, always)
- No new findings at or above `--severity` threshold (default: Medium) in the latest round
- Both reviewers state they have no new findings at the threshold level — with a brief note of what they checked

OR: Max rounds reached → **stopped** (report remaining disagreements)

OR: **Cycling detected** → **stopped** (see below)

**Additional mode-specific requirements:**
- **Diagnose:** both must agree on root cause. Disagreement on cause = not converged, even if no "new" findings.
- **Implement:** per-step convergence. Each step must satisfy the criteria independently. Final verification round on full changeset does not count toward step rounds.

### Cycling detection

Cycling = round N re-argues the same claims as round N-2 with no new evidence. To detect:
1. Compare the finding IDs and evidence cited in round N vs round N-2.
2. If >80% of findings are the same claims with the same evidence (just re-stated), declare cycling.
3. This is a judgment call, not exact string matching. The key question: "Did this round produce any NEW evidence or NEW claims?" If no → cycling.

Stop immediately and report: "Convergence stopped — reviewers are repeating arguments without new evidence. Remaining disagreements require human judgment."

Note: "Assume the other review has at least one error" (the debiasing prompt) does NOT mean reviewers must invent disagreements. If a reviewer checks and finds no errors, they should say so with evidence of what they checked. The debiasing prompt prevents rubber-stamping, not genuine agreement.

---

## Mode: Review

Converge on the quality of a document (spec, plan, design doc, code file).

Reviewers receive: the target document + any `--focus` context.

Fixes are applied directly to the document between rounds. Both reviewers see the updated document in subsequent rounds.

---

## Mode: Diagnose

Converge on the root cause of a bug or unexpected behavior.

### Workflow

1. **Gather context** — read error messages, logs, relevant code. Ask the user for reproduction steps if unclear (block until provided).
2. **Form hypothesis** — state the suspected root cause with evidence.
3. **Run rounds 1-N** on the hypothesis — reviewers stress-test and verify.
4. **Propose fix to user** — present the fix with evidence once converged. The user decides whether to apply it. Do NOT apply fixes to production code without user confirmation in diagnose mode.
5. **Verification round** — after the user approves and the fix is applied, run one more reviewer pass: "Does this fix address the root cause? Any regressions?" This is a bonus round outside the convergence loop — it does not count toward `--rounds`.

---

## Mode: Implement

Converge on a full implementation of a spec or plan. You (Claude) write the code; reviewers verify each step against the spec.

### Workflow

1. **Read the spec and impl plan** — identify the ordered list of steps/stories.
2. **For each step:**
   a. **Implement** — write the code changes for this step.
   b. **Self-check** — run typecheck, lint, tests. Fix any failures.
   c. **Launch both reviewers** — send them the **diff for this step only** (not full files, not cumulative diffs). For files over 100 lines, send only changed hunks with **30 lines** of surrounding context (not 10 — reviewers need enough context to spot aliasing, view relationships, and state set up earlier in the function).
   d. **Run rounds 1-N** per the shared round structure above.
   e. **Step converged** → move to next step.
3. **Final verification** — after all steps, run both reviewers on the full changeset vs. the spec: "Is the spec fully implemented? Any gaps?" This is a single pass, not a convergence loop.
4. **Report.**

### Key rules for implement mode

- **You write the code, reviewers verify.** Don't delegate implementation to subagents.
- **Typecheck/lint/test between steps.** Don't accumulate broken code.
- **NEVER skip reviewer rounds.** Every step must be reviewed before committing. Do not commit steps while "waiting for reviewers on a previous step" — that defeats the purpose. If you catch yourself about to commit without launching reviewers, stop. The whole point of converge-implement is that every step gets reviewed. Skipping rounds to move fast is false economy — bugs that slip through cost more time in CI/production than the review would have taken.
- **Commit after each converged step** (if the user wants — ask on the first step, then follow that preference).
- **If a reviewer finding requires changing the spec or plan**, flag it to the user before proceeding. Don't silently deviate from the spec.
- **If stuck on a step** (reviewers keep finding new issues after max rounds), pause and ask the user.

---

## Reviewer prompting

Both reviewers get these instructions verbatim every round. This is the most important section of the skill — LLM reviewers are highly sensitive to emotional framing and will mirror whatever tone they receive. The goal is **constructive peer review**: rigorous and evidence-based, like two senior engineers reviewing each other's PRs. Direct but respectful. Evidence over opinion. Suggestions over complaints.

```
RULES FOR THIS REVIEW:

You are one of two independent reviewers. Your goal is to make the work better 
through rigorous, evidence-based analysis. This is peer review, not a debate to 
win. Be direct but constructive.

1. EVIDENCE FIRST. Every claim must cite: file path + line number, or a direct 
   quote from the target. A claim without evidence is not a finding — it is 
   speculation. State what the code/doc does, what the spec/intent requires, and 
   the gap between them.

2. SEVERITY. Classify each finding:
   - High: correctness bug, security issue, spec violation, data loss risk
   - Medium: logic gap, missing edge case, unclear behavior, performance issue
   - Low: style, naming, minor readability, non-blocking suggestion

3. NO EMOTIONAL LANGUAGE. Banned words: "clearly", "obviously", "unfortunately",
   "importantly", "crucial". Banned patterns: "I think maybe", "it seems like", 
   "this is wrong". Instead: "Line 42 calls foo(). foo is not defined in this 
   scope." Full stop. LLM reviewers escalate when prompted emotionally — keep 
   all language clinical and neutral.

4. CONSTRUCTIVE. Every finding must include a concrete fix suggestion or a 
   specific question. "Line 42 calls foo() which is undefined — either import 
   it from utils.ts or replace with bar()" is useful. "This is broken" is not.

5. NO DEFENSIVENESS. If the other reviewer contradicted a prior finding:
   - Counter-evidence: "Line 42 shows X, which contradicts reviewer's claim Y"
   - Correction: "Corrected — [prior finding] was wrong because [reason]"
   Never: "I still believe..." or "as I mentioned before..."

6. NO RUBBER-STAMPING. "No issues found" requires MORE rigor than a finding:
   describe what you checked, how you checked it, and why it is correct. If you 
   checked nothing new, say so explicitly. This standard prevents premature 
   convergence.

7. UNCERTAINTY IS OK. Label uncertain findings "[UNCERTAIN]" and state what 
   information would resolve them. Do not suppress findings because you're 
   unsure — flag the uncertainty.

8. CROSS-CRITIQUE (rounds 2+). Assume the other review may contain errors. 
   Independently verify each of the other reviewer's findings. If you agree, 
   explain WHY with evidence — not just "I concur." If you find no errors after
   genuine checking, say so. Do not manufacture disagreements.

9. PROPORTIONAL DEPTH. High findings get full analysis and a fix. Medium get a 
   paragraph. Low get one line. Do not write a paragraph about a naming nit.

10. ALIASING AND VIEWS. When reviewing code that operates on tensors, buffers,
    or arrays: trace each variable back to where it was created. If a variable
    is a view/slice of another (e.g., `y = x[:n]`), flag any operation that
    mutates the underlying buffer while the view is still in use. Common 
    pattern: zeroing a buffer then "copying" from a view of that same buffer 
    copies zeros. This class of bug is invisible in diffs — it requires reading
    the surrounding function context.

11. INITIALIZATION ORDERING. When reviewing code that initializes multiple
    subsystems sequentially: check what mutable state each initialization step 
    leaves behind. Flag cases where step N leaves shared state (flags, buffers,
    descriptors) in a state that corrupts step N+1. Common pattern: a setup 
    function sets a flag for its own use and doesn't reset it, then the next 
    setup function inherits the stale flag.
```

### Symmetric vs asymmetric prompts

- **Round 1 (blind):** Give both reviewers the **same prompt** — identical task, identical rules. Let model diversity (Claude vs GPT) do the work. Same-task independent review produces the cleanest comparison signal.
- **Round 2+ (cross-critique):** Add **light role guidance** to leverage each model's strengths — "pay particular attention to logical coherence and structural issues" for Critic, "pay particular attention to implementation correctness and code-level details" for Codex. They're now engaging with specific findings, not doing independent assessment, so targeted guidance helps.

### Debiasing: independent-then-compare

Anchoring is the primary failure mode of multi-agent review (Liang et al. 2023). The mitigation is structural:

- **Round 1 — Blind:** Both reviewers see ONLY the target (+ pre-scan findings if applicable). No prior findings, no other reviewer's output. Same symmetric prompt.
- **Round 2 — Cross-critique:** Each reviewer receives the OTHER's round 1 findings and must engage with specific claims.
- **Round 3+ — Focused:** Only unresolved findings are sent.

**Never** send both reviewers' findings in the same undifferentiated block. Always label whose findings are whose.

---

## Orchestration

### Launching reviewers

Launch BOTH reviewers in the SAME message using two Agent tool calls with `run_in_background: true`:

```
# In a single message, make both calls:
Agent(subagent_type: "critic", prompt: "...", run_in_background: true)
Agent(subagent_type: "codex-reviewer", prompt: "...", run_in_background: true)
```

**Do NOT launch one blocking and one background** — that serializes execution.

### Waiting for completion

After launching both background agents, you will be automatically notified when each completes. Do not poll or sleep. When both have returned results, proceed to synthesis.

### Error handling

- **Agent timeout or failure:** If one reviewer fails to return results, proceed with the other reviewer's findings only. Note in the report: "Round N: [reviewer] timed out. Findings from [other reviewer] only — treat as unverified."
- **Malformed output:** If a reviewer returns findings without evidence (violating the prompting rules), discard those findings and note: "Round N: [X] findings from [reviewer] discarded — no evidence provided."
- **Both fail:** Report the failure and stop. Do not retry automatically — ask the user.

### Progress signals

After each round, output:

```
--- Round N/M | Findings: X new (H:a M:b L:c) | Cumulative fixed: Y | Status: continuing/converged/stopped ---
```

For implement mode, prefix with the step:
```
--- Step 2/5 | Round N/M | Findings: X new (H:a M:b L:c) | Status: continuing ---
```

### User controls

After each round's progress line, the user may respond. If they do:
- **"skip"** or **"good enough"** → accept current state, move to next step (implement) or stop (review/diagnose)
- **"stop"** → halt convergence entirely, produce report with current findings
- **"override [finding]"** → mark a disagreement as resolved in the user's favor
- **"more rounds"** → increase max rounds by 2 for the current step/target

If the user says nothing (sends a new unrelated message or no message), continue the convergence loop automatically.

### Finding identity and tracking

Assign each finding a stable ID when it first appears: `F1`, `F2`, etc. Use these IDs throughout the convergence — in synthesis tables, cross-critique prompts, and the final report. This prevents:
- **Duplicate detection failure:** paraphrased findings being treated as "new"
- **Cycling false negatives:** re-argued findings not being recognized as the same claim
- **Tracking confusion:** "the finding about foo()" is ambiguous; "F3" is not

When a finding is fixed, note it as "F3: fixed in round N" and do not send it to reviewers again.

### Stale references after edits

When fixes are applied between rounds, line numbers in prior findings may shift. Before sending prior findings to reviewers in the next round:
- Update line numbers if the edit location is known (e.g., you applied the fix, so you know the delta)
- If line numbers can't be reliably updated, replace them with a quote from the relevant code: "the block containing `foo(bar)`" instead of "line 42"
- Never send stale line numbers to reviewers — this causes false confirmations or false disputes

### Context management

Multi-round convergence accumulates context. To prevent prompt bloat:
- **Fixed findings:** One line: "F3: fixed in round N — [description]". Don't include full text or diff.
- **Active findings:** Include full text only for findings still under discussion.
- **Prior-round reviewer output:** Summarize. "Critic round 2: 2 new findings (F5 High, F6 Medium), agreed on F3, disputed F4."
- **Implement mode:** Only the current step's diff. Never cumulative diffs.

---

## Convergence table

After each round, synthesize findings from both reviewers:

```markdown
| ID | Finding | Sev | Critic | Codex | Status |
|----|---------|-----|--------|-------|--------|
| F1 | <claim with evidence> | H/M/L | <position> | <position> | Agree / Disagree / New |
| F2 | ... | ... | ... | ... | Fixed (round N) |
```

Use stable IDs (F1, F2...) throughout. Mark fixed findings so they don't re-enter circulation.

---

## Report format

The orchestrator (you, Claude) generates the report after convergence or stopping.

```markdown
## Convergence Report: <target>

**Mode:** review | diagnose | implement
**Rounds:** N (min: M) | **Severity threshold:** Medium
**Status:** Converged / Stopped (max rounds) / Stopped (cycling) / Stopped (user)

### Accepted findings
1. [H] <finding> — fixed in round N
2. [M] <finding> — fixed in round N

### Remaining disagreements (if any)
1. <topic>: Critic says X (evidence: ...). Codex says Y (evidence: ...).

### Below-threshold findings (not blocking)
1. [L] <finding> — noted, not fixed

### Changes applied
- Round 1: <list>
- Round 2: <list>
```

For implement mode, also include:
```markdown
### Steps completed
1. <step name> — converged in N rounds, M findings fixed
2. ...

### Final verification
- <result>
```
