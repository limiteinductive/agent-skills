# Recipes

These are "agent-friendly" patterns: capture IDs, then operate by ID.

## Create a new tab in the current window and title it

1) Identify a window_id from `wezterm cli list --format json`
2) Spawn into that window
3) Set tab title via the returned pane-id

Example (POSIX shell):

```sh
pane_id="$(wezterm cli spawn --window-id 0 --cwd "$PWD")"
wezterm cli set-tab-title --pane-id "$pane_id" "repo"
```

## Create a 3-pane dev layout in the current tab

Goal:
- keep current pane as "editor" (or whatever is running)
- create right pane for "server"
- create bottom split under right pane for "tests"

Example:

```sh
server_id="$(wezterm cli split-pane --right --percent 35 --cwd "$PWD")"
wezterm cli set-tab-title --pane-id "$server_id" "server"

tests_id="$(wezterm cli split-pane --pane-id "$server_id" --bottom --percent 50 --cwd "$PWD")"
wezterm cli set-tab-title --pane-id "$tests_id" "tests"
```

To run commands, send text including a newline *only if requested*:
```sh
wezterm cli send-text --pane-id "$server_id" 'npm run dev\n'
wezterm cli send-text --pane-id "$tests_id"  'npm test\n'
```

## Move focus predictably

- By id:
  ```sh
  wezterm cli activate-pane --pane-id 123
  ```
- By direction (from current):
  ```sh
  wezterm cli activate-pane-direction Left
  ```
- Discover neighbor id without changing focus:
  ```sh
  neighbor="$(wezterm cli get-pane-direction Right)"
  wezterm cli activate-pane --pane-id "$neighbor"
  ```

## Capture output from a pane for debugging

```sh
wezterm cli get-text --pane-id 123 > /tmp/pane.txt
```

## Rename a workspace

```sh
wezterm cli rename-workspace --workspace default my-project
```

## Resize and zoom

```sh
wezterm cli adjust-pane-size --pane-id 123 Left --amount 5
wezterm cli zoom-pane --pane-id 123 --toggle
```
