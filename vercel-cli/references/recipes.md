# Vercel CLI recipes

## Debug a production Server Components error

**Goal:** Find the actual error message when Next.js shows "An error occurred in the Server Components render" (production builds suppress details).

```sh
# Get full error messages from production
npx vercel logs --level error --expand --no-branch --limit 10
```

The `--expand` flag is critical — without it, you only see truncated messages like "Error: Fa…" instead of the full database error or stack trace.

## Debug a specific deployment

**Goal:** See all logs for a single deployment, including successful requests.

```sh
# Get the deployment URL first
npx vercel ls

# Then get historical logs (--no-follow prevents streaming mode)
npx vercel logs https://my-app-xxxxx.vercel.app --no-follow --expand
```

## Find database connection errors

**Goal:** Identify database connectivity issues in serverless functions.

```sh
npx vercel logs --source serverless --level error --expand --query "database\|connection\|timeout\|ECONNREFUSED" --limit 20
```

## Check if a deployment is healthy after merge

**Goal:** Verify a new production deployment has no errors.

```sh
# List deployments — check latest is "Ready"
npx vercel ls

# Check for any errors in the last 10 minutes
npx vercel logs --level error --since 10m --expand --no-branch
```

## Add a secret environment variable safely

**Goal:** Add a database URL or API key without exposing it in shell history.

```sh
# Use printf (not echo) to avoid trailing newline
printf '%s' "postgresql://user:pass@host:5432/db" | npx vercel env add DATABASE_URL production

# Verify it was added
npx vercel env ls | grep DATABASE_URL
```

## Pull remote env vars for local development

**Goal:** Sync production/preview environment variables to your local `.env.local`.

```sh
npx vercel env pull .env.local --yes
```

## Switch team scope before operating

**Goal:** Ensure you're targeting the correct team before deploying or managing env vars.

```sh
# Check current scope
npx vercel whoami

# Switch to correct team
npx vercel switch my-team

# Now commands target the correct project
npx vercel ls
```

## Search logs for a specific error pattern

**Goal:** Find all occurrences of a specific error across recent logs.

```sh
# Full-text search with JSON output for scripting
npx vercel logs --query "column does not exist" --json --no-branch | jq -r '.message' | head -5

# Or with human-readable expanded output
npx vercel logs --query "timeout" --expand --no-branch --limit 20
```

## Compare edge vs serverless errors

**Goal:** Determine if errors come from middleware (edge) or server-side rendering (serverless).

```sh
# Edge/middleware errors only
npx vercel logs --source edge-middleware --level error --expand --limit 10

# Serverless function errors only
npx vercel logs --source serverless --level error --expand --limit 10
```

Log symbols: `λ` = serverless, `ε` = edge/middleware, `◇` = static.

## Force a redeployment

**Goal:** Redeploy the latest production deployment (e.g., after fixing env vars or database schema).

```sh
# Find the current production deployment
npx vercel ls

# Redeploy it
npx vercel redeploy <deployment-url>
```
