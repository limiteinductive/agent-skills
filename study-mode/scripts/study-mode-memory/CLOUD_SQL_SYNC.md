# Cloud SQL Study Memory Sync

The memory backend is local-first. `record`, `review`, `due`, `show`, `search`, and `profile` keep using `memory.jsonl`; `sync` merges that local event log with a Cloud SQL Postgres table.

## One-time GCP setup

Choose a project, region, instance, database, and user:

```bash
gcloud config set project PROJECT_ID
gcloud sql instances create study-memory \
  --database-version=POSTGRES_16 \
  --region=us-central1 \
  --tier=db-f1-micro
gcloud sql databases create study_memory --instance=study-memory
gcloud sql users create study_user --instance=study-memory --password='REPLACE_ME'
```

Run the Cloud SQL Auth Proxy locally:

```bash
cloud-sql-proxy PROJECT_ID:us-central1:study-memory --port 5432
```

In the shell where study mode runs:

```bash
export STUDY_MODE_DATABASE_URL="postgres://study_user:REPLACE_ME@127.0.0.1:5432/study_memory"
export STUDY_MODE_DEVICE_ID="$(hostname)"
```

## First sync

```bash
scripts/study-mode-memory/run import-local
scripts/study-mode-memory/run sync
scripts/study-mode-memory/run doctor
```

`sync` creates the append-only `study_events` table and indexes if they do not exist. `doctor` validates the local log, sync state, database connectivity, and expected table columns.

## Normal use

Run study commands normally. When switching machines, run:

```bash
scripts/study-mode-memory/run sync
```

Do this before a session to pull remote events, and after a session to push local events. Concurrent edits are safe at the event level: both machines can add events, and sync merges by `event_id` instead of overwriting the file.
