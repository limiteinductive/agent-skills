# WezTerm CLI command reference

Official docs (URLs):
- https://wezterm.org/cli/cli/index.html
- https://wezterm.org/cli/cli/list.html
- https://wezterm.org/cli/cli/list-clients.html
- https://wezterm.org/cli/cli/spawn.html
- https://wezterm.org/cli/cli/split-pane.html
- https://wezterm.org/cli/cli/send-text.html
- https://wezterm.org/cli/cli/activate-pane.html
- https://wezterm.org/cli/cli/activate-pane-direction.html
- https://wezterm.org/cli/cli/activate-tab.html
- https://wezterm.org/cli/cli/set-tab-title.html
- https://wezterm.org/cli/cli/set-window-title.html
- https://wezterm.org/cli/cli/rename-workspace.html
- https://wezterm.org/cli/cli/get-pane-direction.html
- https://wezterm.org/cli/cli/adjust-pane-size.html
- https://wezterm.org/cli/cli/zoom-pane.html
- https://wezterm.org/cli/cli/get-text.html
- https://wezterm.org/cli/cli/kill-pane.html

## Inspect

- List panes (table):
  - wezterm cli list
- List panes (JSON):
  - wezterm cli list --format json

- List clients (table):
  - wezterm cli list-clients
- List clients (JSON):
  - wezterm cli list-clients --format json

## Create

- Spawn a new tab (returns new pane id):
  - wezterm cli spawn
- Spawn into a specific window:
  - wezterm cli spawn --window-id 0
- Spawn a new window in a workspace:
  - wezterm cli spawn --new-window --workspace my-workspace
- Spawn with cwd:
  - wezterm cli spawn --cwd /path/to/repo

- Split an existing pane (returns new pane id):
  - wezterm cli split-pane --pane-id 3
- Split directions:
  - --left | --right | --top | --bottom
- Split with size:
  - --percent 30
  - --cells 40
- Split and set cwd:
  - wezterm cli split-pane --right --cwd /path/to/repo

## Focus / activate

- Focus a specific pane:
  - wezterm cli activate-pane --pane-id 3
- Focus pane by direction:
  - wezterm cli activate-pane-direction Right

- Activate a tab:
  - wezterm cli activate-tab --tab-id 2

## Titles and workspaces

- Set tab title:
  - wezterm cli set-tab-title --tab-id 2 "api"
  - wezterm cli set-tab-title --pane-id 3 "api"
- Set window title:
  - wezterm cli set-window-title --window-id 1 "My Project"
- Rename workspace:
  - wezterm cli rename-workspace --workspace old new

## Interact

- Send text (paste) to current pane:
  - wezterm cli send-text "echo hello"
- Send text to a specific pane:
  - wezterm cli send-text --pane-id 3 "echo hello"
- Read text from a pane:
  - wezterm cli get-text --pane-id 3
- Find adjacent pane id:
  - wezterm cli get-pane-direction --pane-id 3 Right

## Layout adjustments

- Resize a pane:
  - wezterm cli adjust-pane-size --pane-id 3 Right --amount 5
- Zoom/unzoom/toggle:
  - wezterm cli zoom-pane --pane-id 3 --toggle

## Destructive

- Kill a pane immediately (no prompt):
  - wezterm cli kill-pane --pane-id 3
