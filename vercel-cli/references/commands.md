# Vercel CLI command reference

Official docs:
- https://vercel.com/docs/cli

## Auth and identity

- Show current user:
  - vercel whoami
- Login:
  - vercel login
- Switch team/scope:
  - vercel switch <team-name>
- List teams:
  - vercel teams ls

## Project setup

- Link current directory to a Vercel project:
  - vercel link
  - vercel link --yes
- Pull environment variables to local file:
  - vercel env pull .env.local
  - vercel env pull .env.local --yes

## Deployments

- List recent deployments:
  - vercel ls
- Deploy (preview):
  - vercel
- Deploy to production:
  - vercel --prod
- Inspect a deployment:
  - vercel inspect <deployment-url-or-id>
- Promote preview to production:
  - vercel promote <deployment-url>
- Redeploy:
  - vercel redeploy <deployment-url>

## Logs (the most important section for debugging)

- View recent logs (default table, truncated messages):
  - vercel logs
- View logs with full messages (ALWAYS USE THIS FOR DEBUGGING):
  - vercel logs --expand
- Filter by error level with full messages:
  - vercel logs --level error --expand
- Limit number of results:
  - vercel logs --limit 20 --expand
- Filter by time range:
  - vercel logs --since 1h --expand
  - vercel logs --since 30m --until 10m --expand
- Filter by source type:
  - vercel logs --source serverless --expand
  - vercel logs --source edge-function --expand
  - vercel logs --source edge-middleware --expand
- Filter by HTTP status code:
  - vercel logs --status-code 500 --expand
  - vercel logs --status-code 4xx --expand
- Filter by branch:
  - vercel logs --branch main --expand
  - vercel logs --no-branch --expand
- Full-text search:
  - vercel logs --query "timeout" --expand
  - vercel logs --query "column does not exist" --expand
- JSON output for piping:
  - vercel logs --json
  - vercel logs --json --level error | jq '.message'
- Logs for a specific deployment (streams by default):
  - vercel logs <deployment-url>
  - vercel logs <deployment-url> --no-follow --expand
- Logs for a specific request:
  - vercel logs --request-id req_xxxxx --expand
- Combined filters:
  - vercel logs --level error --source serverless --since 2h --expand --no-branch --limit 10

## Environment variables

- List all env vars:
  - vercel env ls
- Add an env var (pipe value to avoid interactive prompt):
  - printf '%s' "my-value" | vercel env add VAR_NAME production
  - printf '%s' "my-value" | vercel env add VAR_NAME preview
  - printf '%s' "my-value" | vercel env add VAR_NAME development
- Remove an env var:
  - vercel env rm VAR_NAME production
- Pull env vars to a local file:
  - vercel env pull .env.local --yes

## Domains

- List domains:
  - vercel domains ls
- Add a domain:
  - vercel domains add example.com
- Inspect domain config:
  - vercel domains inspect example.com

## Destructive (require explicit user confirmation)

- Remove a deployment:
  - vercel rm <deployment-url>
- Remove a project:
  - vercel project rm <project-name>
- Remove an env var:
  - vercel env rm VAR_NAME production
