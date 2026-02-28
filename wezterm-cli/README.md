# WezTerm CLI

Use `wezterm cli` to manipulate an existing WezTerm GUI/mux session: spawn new tabs/windows, split panes, activate/focus targets, set titles, and send text.

## Safety and invariants

- Treat `wezterm cli send-text` as executing user intent via keystrokes/paste. Only send exactly what the user requested, and include a newline only when the user explicitly wants the command to run.
- Never use `wezterm cli kill-pane` unless the user explicitly requests closing/killing a pane (it kills immediately without prompting).
- Prefer deterministic targeting via IDs (`--pane-id`, `--tab-id`, `--window-id`) over relying on whichever pane happens to be focused.

Send-text is basically "remote keyboard telekinesis"; impressive, but you still own what it types.

## How `wezterm cli` connects and targets

### Targeting the correct instance

When multiple WezTerm GUI processes or a multiplexer exist, select the correct instance using one of:
- `--prefer-mux`
- `WEZTERM_UNIX_SOCKET`
- `--class` (to target a GUI window class)

### Targeting panes

Many subcommands accept `--pane-id`. If it is omitted, WezTerm uses:
- `$WEZTERM_PANE` if present
- otherwise the focused pane from the most recently interacted client session

Therefore:
- If the agent runs *inside* the intended WezTerm pane, omitting `--pane-id` is usually fine.
- If the agent runs *outside* WezTerm, or you must be precise, always specify IDs.

## Standard workflow (what to do when this skill triggers)

### Step 1: Preflight

Run:
- `command -v wezterm`
- `wezterm cli --help`

Then check connectivity:
- `wezterm cli list --format json`

If list fails, switch to troubleshooting.

### Step 2: Inspect current state (always do this before mutating)

Prefer JSON output (easy to parse and stable):
- `wezterm cli list --format json`
- `wezterm cli list-clients --format json`

Use this to:
- find the right `workspace`
- identify candidate panes by `title` and/or `cwd`
- capture the exact `window_id`, `tab_id`, `pane_id` you will operate on

### Step 3: Perform the requested operation

Use the smallest set of CLI calls that accomplishes the task:
- create panes/tabs (`spawn`, `split-pane`)
- focus (`activate-pane`, `activate-pane-direction`, `activate-tab`)
- label (`set-tab-title`, `set-window-title`, `rename-workspace`)
- interact (`send-text`, `get-text`)
- adjust layout (`adjust-pane-size`, `zoom-pane`)

### Step 4: Verify result

After changes, re-run:
- `wezterm cli list --format json`

If the goal involved running a command in a pane, optionally capture its output:
- `wezterm cli get-text --pane-id <PANE_ID>`

## Output format when acting as an agent

When executing user requests, respond with:

1) Plan (3–7 bullets)
2) Commands to run (single code block)
3) Expected outcome (concrete checks: updated titles, new panes exist, focused pane id, etc.)
4) Rollback notes when relevant (e.g., "to undo, close the new tab/pane")

Do not add extra explanations unless the user asks.
