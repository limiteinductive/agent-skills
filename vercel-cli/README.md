# Vercel CLI

Use `vercel` (or `npx vercel`) to manage deployments, debug production errors, inspect builds, and manage environment variables for Vercel-hosted projects.

## Safety and invariants

- Never run `vercel env rm` without explicit user confirmation — removing env vars from production can take down a live site.
- Never run `vercel rm` (delete deployment) or `vercel project rm` without explicit user confirmation.
- Prefer `--yes` only for non-destructive operations (e.g., `vercel link --yes`, `vercel env pull --yes`).
- Always check which **scope/team** you are operating under before mutating anything (`vercel whoami`, `vercel switch`).
- When the project is not linked, run `vercel link` first. Many commands silently fail or target the wrong project without linking.

## Key concept: log message truncation

The default `vercel logs` output **truncates log messages** to fit a table format. This makes debugging production errors nearly impossible with the default view.

**Always use `--expand` (or `--json`) to see full log messages.** This is the single most important flag for debugging.

```
# BAD — message truncated:
vercel logs --level error
# Shows: "Error: Fa…"

# GOOD — full message visible:
vercel logs --level error --expand
# Shows: "Error: Failed query: select ... column "foo" does not exist"
```

## Scope and project context

### Teams / Scopes

Vercel CLI operations are scoped to a team. Check and switch:
- `vercel whoami` — show current user
- `vercel switch <team>` — switch to a specific team
- `vercel teams ls` — list available teams

### Project linking

Most commands require the project to be linked:
- `vercel link` — interactively link current directory to a project
- `vercel link --yes` — auto-confirm linking

If a command returns `"not_linked"`, run `vercel link` first.

## Standard workflow (what to do when this skill triggers)

### Step 1: Preflight

Verify CLI is available and authenticated:
- `npx vercel whoami`
- `npx vercel ls` (lists recent deployments, confirms project access)

If not linked, run `npx vercel link --yes`.

### Step 2: Inspect current state

Check deployments and their status:
- `npx vercel ls` — list recent deployments with status, environment, age
- `npx vercel inspect <deployment-url-or-id>` — detailed info for one deployment

### Step 3: Debug or perform the requested operation

**For debugging production errors** (most common use case):

1. Get recent error logs with full messages:
   ```
   npx vercel logs --level error --expand --no-branch --limit 10
   ```

2. If you need logs for a specific deployment:
   ```
   npx vercel logs <deployment-url> --no-follow --expand
   ```

3. Search logs by keyword:
   ```
   npx vercel logs --query "timeout" --json | jq '.message'
   ```

4. Filter by source (serverless vs edge):
   ```
   npx vercel logs --source serverless --level error --expand
   ```

**For deployment management:**
- `npx vercel` — deploy from current directory
- `npx vercel --prod` — deploy to production
- `npx vercel promote <deployment-url>` — promote a preview to production

**For environment variables:**
- `npx vercel env ls` — list all env vars
- `npx vercel env pull .env.local --yes` — pull remote env vars to local file
- `printf '%s' "value" | npx vercel env add VAR_NAME production` — add env var (use printf, not echo, to avoid trailing newline)

### Step 4: Verify result

After deploying or changing config:
- `npx vercel ls` — confirm new deployment status is "Ready"
- `npx vercel logs --level error --expand --limit 5` — check for new errors

## Log source symbols

Vercel logs prefix each entry with a symbol indicating where it ran:
- `λ` — Serverless function (Node.js runtime)
- `ε` — Edge function or middleware
- `◇` — Static file or external rewrite

This helps distinguish middleware errors from server-side rendering errors.

## Output format when acting as an agent

When executing user requests, respond with:

1. Plan (3–7 bullets)
2. Commands to run (single code block)
3. Expected outcome (concrete checks: deployment status, error resolved, env var set)
4. Rollback notes when relevant (e.g., "to undo, re-add the env var")

Do not add extra explanations unless the user asks.

## Common gotchas

- **Branch filtering**: `vercel logs` auto-detects your git branch and filters to it. Use `--no-branch` to see logs from all branches, or `--branch main` to explicitly target production.
- **Env var newlines**: Use `printf '%s' "value" | vercel env add` not `echo` — echo adds a trailing newline that breaks secrets.
- **Pooler hostnames**: Supabase pooler uses `aws-<region>.pooler.supabase.com`, not `db.<ref>.supabase.co`. The latter won't resolve from Vercel's network.
- **`--follow` is implicit**: When you pass a deployment URL/ID, `vercel logs` enters streaming mode. Use `--no-follow` for historical logs.
