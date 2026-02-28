Audit agent-facing docs and CLI/tool surfaces against AX principles and A/B test every proposed fix with sub-agents before recommending it. Use when agent behavior is wrong due to missing/unclear docs or poor CLI ergonomics (cryptic errors, missing flags, inconsistent interfaces).

Arguments: $ARGUMENTS (optional problem description, or empty for general audit)

---

# AX — Agent Experience Audit & Test

Audit agent-facing documentation **and CLI/tool surfaces** against 8 AX principles, propose concrete fixes, and **A/B test high-impact fixes** with Haiku sub-agents before recommending them. Only validated improvements make the final report.

## Arguments

- `$ARGUMENTS` — a description of the AX problem (e.g., "agents keep running the wrong build command"). If empty, run a general audit on all agent-facing docs.

## Workflow

### Phase 1: AUDIT — Discover, ground-truth, and score docs

#### 1a. Establish ground truth

Before scoring docs, derive the canonical workflow from automation sources:
- CI pipelines (`.github/workflows/*`, `.gitlab-ci.yml`, etc.)
- `Taskfile.yml`, `Makefile`, `justfile`
- `pyproject.toml` / `package.json` scripts
- `Dockerfile`, `docker-compose.yml`, `devcontainer.json`
- `.env.example`, lock files (`uv.lock`, `pnpm-lock.yaml`, `poetry.lock`)

This ground truth is used to judge whether docs (and later, sub-agent plans) are correct. If docs recommend commands that differ from what automation actually runs, that's a finding.

#### 1b. Find agent-facing files

If `$ARGUMENTS` is provided, interpret the problem and explore the project to find files an agent would encounter when tackling that problem. Always check for:
- `AGENTS.md`, `CLAUDE.md`, `README.md` at repo root
- `Taskfile.yml`, `Makefile`, `pyproject.toml`, `package.json` (build/setup context)
- `scripts/*`, `.github/workflows/*`
- Any files referenced from the above

If `$ARGUMENTS` is empty, audit all agent-facing docs found in the repo.

#### 1b-ext. Enumerate CLI surfaces

Discover CLIs that agents interact with:
- **Sources**: commands in AGENTS.md/CLAUDE.md code blocks, Taskfile/Makefile task bodies, pyproject.toml `[project.scripts]`, package.json `scripts`, executables in `scripts/` directory
- **Introspection**: run `<tool> --help` and `<tool> <subcommand> --help` recursively (cap at 3 levels depth) to build a surface map per tool:
  - Subcommand tree
  - Flags per subcommand (required vs optional)
  - Output format flags (`--output`, `--format`, `-o json`)
  - Safety flags (`--dry-run`, `-y`, `--force`)
  - Deprecated aliases (look for "deprecated", "removed", "use X instead" in help text)

This introspection output becomes additional ground truth alongside CI/automation.

#### 1c. Score against AX principles

Read each file and evaluate it against all 8 principles below. For each principle, assign a rating: **PASS**, **WARN**, or **FAIL**. Score per-file where relevant — problems are file-local even if the scorecard is global.

**The 8 AX Principles:**

| # | Principle | FAIL when... |
|---|-----------|--------------|
| 1 | Explicitness over convention | A non-standard workflow isn't called out explicitly |
| 2 | Fail fast with clear recovery | Errors lack concrete fix commands OR success signals to confirm recovery worked |
| 3 | Minimize context rot | Docs add tokens that don't earn their keep — every line competes with the actual task context window |
| 4 | Structured over unstructured | Important info is buried in prose instead of headers, tables, or code blocks |
| 5 | Consistent patterns | Naming or formatting conventions shift within the doc |
| 6 | Complete context at point of need | Critical runnable commands are missing inline (use progressive disclosure: inline the minimum + link deeper detail) |
| 7 | Guard rails over documentation | Says "don't do X" but X would succeed silently — a pre-commit hook or validation would be better |
| 8 | CI parity / single source of truth | Docs diverge from CI/automation or recommend commands not used by automation |

**CLI-specific checks** (concrete readings of existing principles applied to CLI surfaces):

| Check | Maps to | FAIL when... |
|-------|---------|--------------|
| Machine-readable output | #4 Structured | No `--output=json` (or equivalent) on commands returning structured data |
| Long-form input path | #6 Complete context | Command accepts freeform text but has no `--file` flag — forces shell-escaping hacks |
| Deprecation surfacing | #8 CI parity | Deprecated aliases aren't surfaced in `--help` with their replacement, or docs still reference deprecated forms |

**Tension resolution rules:**
- **#3 vs #6**: prefer progressive disclosure — inline the minimum runnable commands + one-line explanation; link deeper detail. If inlining would add >200 tokens, use a short snippet + pointer.
- **#7 (guard rails)**: evaluated via static reasoning ("does this actually prevent silent success?"), not via A/B testing. Propose the guardrail, explain what it prevents, but don't try to A/B test code changes.

### Phase 2: PROPOSE — Draft and triage fixes

For each WARN or FAIL, draft a concrete fix: an addition, edit, or removal. Each fix must include:
- Which principle it addresses
- The exact change (diff-style: what to add, edit, or remove)
- Estimated token impact (+N or -N tokens)

**CLI fix classification** — CLI-related findings split into two types:
- **Doc-side fix**: docs reference a CLI incorrectly (e.g., use deprecated command, wrong flags). Normal doc diff — goes through A/B testing like any other doc fix.
- **CLI-side finding**: the CLI itself is the problem (missing flag, bad error message, no machine-readable output). Cannot be A/B tested via docs. Report as a static recommendation with: current behavior, recommended CLI change, and interim doc workaround until the CLI is fixed.

**Triage gate**: classify each fix as **high-impact** or **low-impact**.
- **High-impact**: fixes that change what commands an agent would run, what order it would follow, or whether it would recover from errors. These get A/B tested.
- **Low-impact**: formatting, minor wording, structural cleanup. These are recommended directly without A/B testing, grouped as a bundle in the report.

### Phase 3: TEST — A/B test high-impact fixes

For each **high-impact** fix, run a docs-only A/B test. This tests documentation quality in isolation — sub-agents cannot compensate for bad docs by searching the repo.

#### Test setup

1. **Derive a test task** from the problem description (or from the doc's subject matter if no argument was given). Examples:
   - Setup docs → "You need to install and build this project. What commands do you run?"
   - PR workflow docs → "You need to create a PR for a small bug fix. Walk through your process."
   - Testing docs → "You need to run the test suite. What's your approach?"

   For **doc-side CLI fixes**, use invocation-correctness tasks that test whether the agent gets the exact CLI invocation right:
   - "Delete cluster `alice-test` in namespace `training`. What exact command?" (tests flag correctness)
   - "Deploy a training run with `base.yaml` and `Dockerfile`. What's the exact invocation?" (tests required flag discovery)

   Evaluate sub-agent outputs against `--help` ground truth for: flag correctness, required flag completeness, deprecation avoidance, and safety flag usage.

2. **Prepare two doc bundles:**
   - **Agent A docs**: the original docs with THIS fix applied
   - **Agent B docs**: the original docs unchanged (for edits/additions) or the original docs with the section present (for removals)

3. **Spawn two sub-agents** using the Task tool:
   - `subagent_type: "general-purpose"`, `model: "haiku"`, `max_turns: 5`
   - Both get: the doc bundle injected into their prompt + the same test task
   - **No repo access** — sub-agents must rely solely on the injected docs (do NOT tell them to use Glob, Grep, or Read)
   - Both must produce: exact commands (with flags, working directory), expected success signals, and one likely failure + recovery path they anticipate from the docs
   - **Launch both agents in parallel** (two Task calls in one message)

4. **Evaluate results** against ground truth (you, as Opus, judge both plans):
   - Correctness vs ground truth (right commands, right ordering)
   - Confidence (does the agent hedge or branch unnecessarily?)
   - Completeness (success signals, failure anticipation)
   - Clear attributable difference to the doc change

5. **Verdict per fix:**
   - **VALIDATED** — Agent A clearly outperformed Agent B on the relevant behavior
   - **INCONCLUSIVE** — No measurable difference between the two plans
   - **REJECTED** — Agent A performed worse, or the fix added noise without helping

#### Integration test

After individual A/B tests, run one final A/B test:
- **Agent A**: all VALIDATED fixes applied together
- **Agent B**: original docs unchanged

Report whether the combined changes remain beneficial or introduce conflicts/confusion.

### Phase 4: REPORT — Output structured results

Present findings in this format:

```
# AX Audit Report

**Project:** <name>  |  **Files:** <list>  |  **Est. tokens:** <total token count across audited files>

## Ground Truth

| Step | Canonical command | Source |
|------|-------------------|--------|
| Build | <command> | <Taskfile / CI / ...> |
| Test | <command> | ... |
| ... | ... | ... |

## Scorecard

| # | Principle | Rating | Detail |
|---|-----------|--------|--------|
| 1 | Explicitness over convention | PASS/WARN/FAIL | one-line explanation |
| 2 | Fail fast with clear recovery | ... | ... |
| 3 | Minimize context rot | ... | ... |
| 4 | Structured over unstructured | ... | ... |
| 5 | Consistent patterns | ... | ... |
| 6 | Complete context at point of need | ... | ... |
| 7 | Guard rails over documentation | ... | ... |
| 8 | CI parity / single source of truth | ... | ... |

## Per-File Findings

| File | Key issues | Highest-impact fixes |
|------|-----------|----------------------|
| ... | ... | ... |

## CLI Surface Inventory

| Tool | Subcommands audited | `--output=json` | `--dry-run` | Deprecations found |
|------|--------------------:|:---------------:|:-----------:|:------------------:|
| <tool> | N | yes/no | yes/no | N |

## CLI Findings

### CLI-side recommendations (static — not A/B tested)

These are problems with the CLI itself, not the docs. Each includes current behavior, recommended CLI change, and an interim doc workaround.

#### 1. <title>
- **Principle:** #N — <principle name>
- **Current behavior:** <what happens now>
- **Recommended CLI change:** <what the tool maintainer should change>
- **Interim doc workaround:** <what to add to docs until the CLI is fixed>

### Doc-side CLI fixes

Doc-side CLI fixes (e.g., docs referencing deprecated commands) go through normal A/B testing and appear in the Validated/Rejected sections below.

## Validated Recommendations

### 1. <title of change>
- **Principle:** #N — <principle name>
- **Change:**
  ```diff
  <exact diff showing what to add/edit/remove>
  ```
- **A/B Result:** VALIDATED — Agent A correctly did X; Agent B failed with Y
- **Token impact:** +N / -N tokens

### 2. ...

(repeat for each VALIDATED fix)

## Low-Impact Changes (not A/B tested)

| Change | Principle | Token impact |
|--------|-----------|--------------|
| <short description> | #N | +/- N |

## Rejected / Inconclusive Changes

| Change | Verdict | Why |
|--------|---------|-----|
| <short description> | INCONCLUSIVE/REJECTED | <one-line explanation> |

## Integration Test Result

<Did the combined VALIDATED fixes hold up? Any interaction effects?>

## Token Budget

| Metric | Value |
|--------|-------|
| Current total | ~N tokens |
| After recommendations | ~M tokens |
| Delta | +/- N tokens |
| Density | % lines that are runnable commands, decision rules, or structured references |
```

## Important Notes

- This skill is **read-only** — it never modifies files, only reports recommendations
- Sub-agents are **docs-only** — no repo access, ensuring the A/B test measures doc quality, not agent search skill
- Sub-agents use **Haiku** with a **5 turn cap** for cost control
- Always launch both A/B agents **in parallel** (two Task calls in one message)
- If `$ARGUMENTS` is provided, the test task should directly reflect the described problem
- If a fix is a **removal** (cutting verbose content), still A/B test it — Agent A gets the trimmed version, Agent B gets the original
- Token estimates: count characters in agent-facing sections (exclude badges, changelogs, non-agent content), divide by 4
- Density: % of lines that are runnable commands, decision rules, or structured references (tables, code blocks)
- Be conservative with VALIDATED — only mark a fix as validated when there is a **clear, attributable difference** between Agent A and Agent B plans
- Inject doc content directly into sub-agent prompts (never tell them to read files — this is the controlled variable)
- **Principle #7 (guard rails)**: propose guardrails with static reasoning; don't A/B test them (plan-only agents can't trigger hooks)
- **Ground truth is authoritative**: if docs and automation disagree, the fix should align docs with automation, not the other way around
- **CLI-side findings are static recommendations** — they report problems with the tool itself (missing flags, bad errors) and cannot be A/B tested. They include an interim doc workaround so agents can cope until the CLI is fixed
- **CLI introspection is capped at 3 levels** of subcommand depth to avoid runaway `--help` calls
