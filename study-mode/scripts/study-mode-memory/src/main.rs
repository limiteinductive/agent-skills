use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::Duration as StdDuration;
use uuid::Uuid;

const ALGORITHM_VERSION: &str = "study-srs-v1";
const EVENT_ID_NAMESPACE: Uuid = uuid::uuid!("1b8c6d23-f55f-4a0a-a49e-f1d8272c106d");
const DATABASE_URL_ENV: &str = "STUDY_MODE_DATABASE_URL";
const DEVICE_ID_ENV: &str = "STUDY_MODE_DEVICE_ID";
const DB_CONFIG_NAME_ENV: &str = "STUDY_MODE_DB_CONFIG_NAME";
const REQUIRED_COLUMNS: &[&str] = &[
    "event_id",
    "created_at",
    "received_at",
    "device_id",
    "concept_id",
    "topic",
    "concept",
    "entry_json",
];

#[derive(Parser, Debug)]
#[command(name = "study-mode-memory")]
#[command(about = "Persistent Study Mode memory and spaced repetition backend.")]
struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    store: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print the default memory store path.
    Path,
    /// Record a new concept or non-review learning event.
    Record(RecordArgs),
    /// Record a retrieval-practice review event.
    Review(ReviewArgs),
    /// Show due spaced-repetition review items.
    Due(DueArgs),
    /// Show recent study memory entries.
    Show(ShowArgs),
    /// Search study memory entries.
    Search(SearchArgs),
    /// Summarize recent study memory and calibration signals.
    Profile(ProfileArgs),
    /// Merge local memory with the configured Cloud SQL Postgres database.
    Sync,
    /// Check local memory and Cloud SQL sync configuration.
    Doctor,
    /// Rewrite local memory entries with stable event IDs.
    ImportLocal,
}

#[derive(clap::Args, Debug)]
struct RecordArgs {
    #[arg(long)]
    topic: String,
    #[arg(long)]
    concept: String,
    #[arg(long)]
    summary: String,
    #[arg(long, default_value = "introduced")]
    status: Status,
    #[arg(long)]
    evidence: Option<String>,
    #[arg(long)]
    next_step: Option<String>,
    #[arg(long)]
    difficulty: Option<DifficultySignal>,
    #[arg(long)]
    confidence: Option<u8>,
    #[arg(long)]
    pace: Option<Pace>,
    #[arg(long)]
    user_signal: Option<String>,
    #[arg(long = "preference")]
    preferences: Vec<String>,
    #[arg(long)]
    review_after_days: Option<i64>,
    #[arg(long)]
    next_review_at: Option<DateTime<Utc>>,
}

#[derive(clap::Args, Debug)]
struct ReviewArgs {
    #[arg(long)]
    topic: String,
    #[arg(long)]
    concept: String,
    #[arg(long)]
    summary: String,
    #[arg(long)]
    answer_quality: f64,
    #[arg(long, default_value_t = 0)]
    hints_used: u8,
    #[arg(long)]
    retrieval_depth: RetrievalDepth,
    #[arg(long)]
    evidence: Option<String>,
    #[arg(long)]
    next_step: Option<String>,
    #[arg(long)]
    confidence: Option<u8>,
    #[arg(long)]
    pace: Option<Pace>,
    #[arg(long)]
    user_signal: Option<String>,
    #[arg(long = "misconception")]
    misconceptions: Vec<String>,
    #[arg(long = "preference")]
    preferences: Vec<String>,
}

#[derive(clap::Args, Debug)]
struct DueArgs {
    #[arg(long)]
    topic: Option<String>,
    #[arg(long, default_value_t = 10)]
    limit: usize,
    #[arg(long)]
    all: bool,
}

#[derive(clap::Args, Debug)]
struct ShowArgs {
    #[arg(long)]
    topic: Option<String>,
    #[arg(long, default_value_t = 20)]
    limit: usize,
}

#[derive(clap::Args, Debug)]
struct SearchArgs {
    #[arg(long)]
    query: String,
    #[arg(long, default_value_t = 20)]
    limit: usize,
}

#[derive(clap::Args, Debug)]
struct ProfileArgs {
    #[arg(long)]
    topic: Option<String>,
    #[arg(long, default_value_t = 50)]
    limit: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
enum Status {
    Introduced,
    Practiced,
    Mastered,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
enum DifficultySignal {
    TooEasy,
    JustRight,
    TooHard,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
enum Pace {
    Slower,
    Steady,
    Faster,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
enum RetrievalDepth {
    Recognition,
    Recall,
    Explanation,
    Application,
    Transfer,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RecallResult {
    Again,
    Hard,
    Good,
    Easy,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Entry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    event_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    algorithm_version: String,
    concept_id: String,
    topic: String,
    concept: String,
    summary: String,
    status: Status,
    memory: MemoryState,
    next_review_at: DateTime<Utc>,
    review_after_days: f64,
    evidence: Option<String>,
    next_step: Option<String>,
    difficulty_signal: Option<DifficultySignal>,
    confidence: Option<u8>,
    pace: Option<Pace>,
    user_signal: Option<String>,
    preferences: Vec<String>,
    review: Option<ReviewSignal>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct MemoryState {
    difficulty: f64,
    stability_days: f64,
    review_count: u32,
    lapse_count: u32,
    last_review_at: Option<DateTime<Utc>>,
    retrievability: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ReviewSignal {
    recall_result: RecallResult,
    answer_quality: f64,
    hints_used: u8,
    retrieval_depth: RetrievalDepth,
    misconceptions: Vec<String>,
}

#[derive(Clone, Debug)]
struct EntryRecord {
    entry: Entry,
    had_event_id: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct SyncState {
    device_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    db_config_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_successful_sync_at: Option<DateTime<Utc>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let store = cli.store.unwrap_or_else(default_store_path);

    match cli.command {
        Command::Path => println!("{}", store.display()),
        Command::Record(args) => cmd_record(&store, args)?,
        Command::Review(args) => cmd_review(&store, args)?,
        Command::Due(args) => cmd_due(&store, args)?,
        Command::Show(args) => cmd_show(&store, args)?,
        Command::Search(args) => cmd_search(&store, args)?,
        Command::Profile(args) => cmd_profile(&store, args)?,
        Command::Sync => cmd_sync(&store).await?,
        Command::Doctor => cmd_doctor(&store).await?,
        Command::ImportLocal => cmd_import_local(&store)?,
    }

    Ok(())
}

fn default_store_path() -> PathBuf {
    let root = env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = env::var_os("HOME").unwrap_or_else(|| ".".into());
            PathBuf::from(home).join(".codex")
        });
    root.join("study-mode").join("memory.jsonl")
}

fn cmd_record(store: &PathBuf, args: RecordArgs) -> Result<()> {
    validate_confidence(args.confidence)?;
    let entries = read_entries(store)?;
    let previous = latest_for_concept(&entries, &args.topic, &args.concept);
    let now = Utc::now();
    let memory = previous
        .map(|entry| entry.memory.clone())
        .unwrap_or_else(|| initial_memory(args.difficulty, args.confidence));

    let due = if let Some(explicit) = args.next_review_at {
        explicit
    } else {
        let days = args
            .review_after_days
            .map(|days| days.max(1) as f64)
            .unwrap_or_else(|| interval_for_status(args.status, args.difficulty, args.confidence));
        now + duration_from_days(days)
    };

    let entry = Entry {
        event_id: Some(Uuid::new_v4()),
        created_at: now,
        algorithm_version: ALGORITHM_VERSION.to_string(),
        concept_id: concept_id(&args.topic, &args.concept),
        topic: args.topic.trim().to_string(),
        concept: args.concept.trim().to_string(),
        summary: args.summary.trim().to_string(),
        status: args.status,
        memory,
        next_review_at: due,
        review_after_days: days_between(now, due),
        evidence: trim_opt(args.evidence),
        next_step: trim_opt(args.next_step),
        difficulty_signal: args.difficulty,
        confidence: args.confidence,
        pace: args.pace,
        user_signal: trim_opt(args.user_signal),
        preferences: clean_vec(args.preferences),
        review: None,
    };

    append_entry(store, &entry)?;
    println!(
        "Recorded concept {} due {}",
        entry.concept_id, entry.next_review_at
    );
    Ok(())
}

fn cmd_review(store: &PathBuf, args: ReviewArgs) -> Result<()> {
    if !(0.0..=1.0).contains(&args.answer_quality) {
        return Err(anyhow!("--answer-quality must be between 0.0 and 1.0"));
    }
    validate_confidence(args.confidence)?;

    let entries = read_entries(store)?;
    let previous = latest_for_concept(&entries, &args.topic, &args.concept);
    let now = Utc::now();
    let prior_memory = previous
        .map(|entry| entry.memory.clone())
        .unwrap_or_else(|| initial_memory(None, args.confidence));
    let rating = infer_recall_result(args.answer_quality, args.hints_used, args.retrieval_depth);
    let memory = update_memory(
        &prior_memory,
        rating,
        args.answer_quality,
        args.hints_used,
        args.retrieval_depth,
        args.confidence,
        now,
    );
    let status = status_from_review(rating, args.retrieval_depth, args.answer_quality);
    let next_review_at = now + duration_from_days(memory.stability_days);

    let entry = Entry {
        event_id: Some(Uuid::new_v4()),
        created_at: now,
        algorithm_version: ALGORITHM_VERSION.to_string(),
        concept_id: concept_id(&args.topic, &args.concept),
        topic: args.topic.trim().to_string(),
        concept: args.concept.trim().to_string(),
        summary: args.summary.trim().to_string(),
        status,
        review_after_days: memory.stability_days,
        next_review_at,
        memory: memory.clone(),
        evidence: trim_opt(args.evidence),
        next_step: trim_opt(args.next_step),
        difficulty_signal: difficulty_signal_from_rating(rating),
        confidence: args.confidence,
        pace: args.pace,
        user_signal: trim_opt(args.user_signal),
        preferences: clean_vec(args.preferences),
        review: Some(ReviewSignal {
            recall_result: rating,
            answer_quality: round2(args.answer_quality),
            hints_used: args.hints_used,
            retrieval_depth: args.retrieval_depth,
            misconceptions: clean_vec(args.misconceptions),
        }),
    };

    append_entry(store, &entry)?;
    println!(
        "Recorded review {} as {:?}; next review {}",
        entry.concept_id, rating, entry.next_review_at
    );
    Ok(())
}

fn cmd_due(store: &PathBuf, args: DueArgs) -> Result<()> {
    let now = Utc::now();
    let entries = read_entries(store)?;
    let mut latest = latest_entries(
        entries
            .iter()
            .filter(|entry| topic_matches(entry, args.topic.as_deref())),
    );
    latest.retain(|entry| args.all || entry.next_review_at <= now);
    latest.sort_by_key(|entry| due_priority(entry));

    if latest.is_empty() {
        println!("No due review items.");
        return Ok(());
    }

    for entry in latest.iter().take(args.limit) {
        print_due(entry, now);
    }
    Ok(())
}

fn cmd_show(store: &PathBuf, args: ShowArgs) -> Result<()> {
    let entries = read_entries(store)?;
    let mut filtered: Vec<&Entry> = entries
        .iter()
        .filter(|entry| topic_matches(entry, args.topic.as_deref()))
        .collect();
    filtered.sort_by_key(|entry| entry_order(entry));

    if filtered.is_empty() {
        println!("No matching study memory entries.");
        return Ok(());
    }

    let start = filtered.len().saturating_sub(args.limit);
    for entry in &filtered[start..] {
        print_entry(entry);
    }
    Ok(())
}

fn cmd_search(store: &PathBuf, args: SearchArgs) -> Result<()> {
    let query = args.query.to_lowercase();
    let entries = read_entries(store)?;
    let mut filtered: Vec<&Entry> = entries
        .iter()
        .filter(|entry| searchable_text(entry).contains(&query))
        .collect();
    filtered.sort_by_key(|entry| entry_order(entry));

    if filtered.is_empty() {
        println!("No matching study memory entries.");
        return Ok(());
    }

    let start = filtered.len().saturating_sub(args.limit);
    for entry in &filtered[start..] {
        print_entry(entry);
    }
    Ok(())
}

fn cmd_profile(store: &PathBuf, args: ProfileArgs) -> Result<()> {
    let entries = read_entries(store)?;
    let mut filtered: Vec<&Entry> = entries
        .iter()
        .filter(|entry| topic_matches(entry, args.topic.as_deref()))
        .collect();
    filtered.sort_by_key(|entry| entry_order(entry));

    if filtered.is_empty() {
        println!("No matching study profile yet.");
        return Ok(());
    }

    let start = filtered.len().saturating_sub(args.limit);
    let recent = &filtered[start..];
    let mut statuses: BTreeMap<String, usize> = BTreeMap::new();
    let mut ratings: BTreeMap<String, usize> = BTreeMap::new();
    let mut paces: BTreeMap<String, usize> = BTreeMap::new();
    let mut confidences = Vec::new();
    let mut preferences = Vec::new();
    let mut signals = Vec::new();
    let mut next_steps = Vec::new();

    for entry in recent {
        *statuses
            .entry(format!("{:?}", entry.status).to_lowercase())
            .or_default() += 1;
        if let Some(review) = &entry.review {
            *ratings
                .entry(format!("{:?}", review.recall_result).to_lowercase())
                .or_default() += 1;
        }
        if let Some(pace) = entry.pace {
            *paces
                .entry(format!("{:?}", pace).to_lowercase())
                .or_default() += 1;
        }
        if let Some(confidence) = entry.confidence {
            confidences.push(confidence as f64);
        }
        push_unique_all(&mut preferences, &entry.preferences);
        push_unique_opt(&mut signals, entry.user_signal.as_ref());
        push_unique_opt(&mut next_steps, entry.next_step.as_ref());
    }

    let due_count = latest_entries(filtered.iter().copied())
        .iter()
        .filter(|entry| entry.next_review_at <= Utc::now())
        .count();

    println!(
        "Study profile for {}: {} recent entries",
        args.topic.as_deref().unwrap_or("all topics"),
        recent.len()
    );
    println!("- statuses: {:?}", statuses);
    if !ratings.is_empty() {
        println!("- recall results: {:?}", ratings);
    }
    if !confidences.is_empty() {
        let avg = confidences.iter().sum::<f64>() / confidences.len() as f64;
        println!("- confidence: average {:.1}/5", avg);
    }
    if !paces.is_empty() {
        println!("- pace: {:?}", paces);
    }
    if due_count > 0 {
        println!("- due reviews: {}", due_count);
    }
    print_joined("- observed preferences", &preferences, 5);
    print_joined("- recent user signals", &signals, 5);
    print_joined("- next steps", &next_steps, 3);
    Ok(())
}

async fn cmd_sync(store: &PathBuf) -> Result<()> {
    let pool = connect_database().await?;
    ensure_schema(&pool).await?;

    let entries = read_entries(store)?;
    let mut local_ids = HashSet::new();
    let mut duplicate_local_events = 0usize;
    for entry in &entries {
        let event_id = required_event_id(entry)?;
        if !local_ids.insert(event_id) {
            duplicate_local_events += 1;
        }
    }

    let mut state = load_sync_state(store)?;
    let device_id = resolve_device_id(&state)?;
    state.device_id = device_id.clone();
    state.db_config_name = database_config_name();

    let remote_ids = fetch_remote_event_ids(&pool).await?;
    let mut pushed = 0usize;
    for entry in &entries {
        let event_id = required_event_id(entry)?;
        if !remote_ids.contains(&event_id) {
            insert_remote_event(&pool, &device_id, entry).await?;
            pushed += 1;
        }
    }

    let remote_events = fetch_remote_events(&pool).await?;
    let mut pulled = Vec::new();
    for (event_id, entry_json) in remote_events {
        if local_ids.contains(&event_id) {
            continue;
        }
        let entry = entry_from_remote_event(event_id, entry_json)?;
        if local_ids.insert(event_id) {
            pulled.push(entry);
        }
    }
    append_entries(store, &pulled)?;

    state.last_successful_sync_at = Some(Utc::now());
    save_sync_state(store, &state)?;

    if duplicate_local_events > 0 {
        println!(
            "Synced study memory: pushed {}, pulled {}, duplicate local events skipped {}",
            pushed,
            pulled.len(),
            duplicate_local_events
        );
    } else {
        println!(
            "Synced study memory: pushed {}, pulled {}",
            pushed,
            pulled.len()
        );
    }
    Ok(())
}

async fn cmd_doctor(store: &PathBuf) -> Result<()> {
    let records = read_entry_records(store)?;
    let missing_event_ids = records.iter().filter(|record| !record.had_event_id).count();
    let mut event_ids = HashSet::new();
    let duplicate_event_ids = records
        .iter()
        .filter_map(|record| record.entry.event_id)
        .filter(|event_id| !event_ids.insert(*event_id))
        .count();

    println!("Local store: {}", store.display());
    println!("- entries: {}", records.len());
    println!("- legacy entries without event_id: {}", missing_event_ids);
    println!("- duplicate event_ids: {}", duplicate_event_ids);

    let state_path = sync_state_path(store)?;
    let state = load_sync_state(store)?;
    println!("Sync state: {}", state_path.display());
    if !state.device_id.is_empty() {
        println!("- device_id: {}", state.device_id);
    }
    if let Some(last_sync) = state.last_successful_sync_at {
        println!("- last successful sync: {}", last_sync);
    }

    let pool = connect_database().await?;
    let missing_columns = missing_schema_columns(&pool).await?;
    if !missing_columns.is_empty() {
        return Err(anyhow!(
            "study_events schema is missing required columns: {}",
            missing_columns.join(", ")
        ));
    }

    let remote_count: i64 = sqlx::query_scalar("select count(*) from study_events")
        .fetch_one(&pool)
        .await
        .context("failed to count remote study_events")?;
    println!("Database: connected; study_events rows: {}", remote_count);
    Ok(())
}

fn cmd_import_local(store: &PathBuf) -> Result<()> {
    if !store.exists() {
        println!("No local study memory file at {}", store.display());
        return Ok(());
    }

    let records = read_entry_records(store)?;
    let assigned = records.iter().filter(|record| !record.had_event_id).count();
    let entries: Vec<Entry> = records.into_iter().map(|record| record.entry).collect();
    rewrite_entries(store, &entries)?;
    println!(
        "Normalized {} local study entries; assigned {} legacy event_ids",
        entries.len(),
        assigned
    );
    Ok(())
}

fn read_entries(store: &PathBuf) -> Result<Vec<Entry>> {
    Ok(read_entry_records(store)?
        .into_iter()
        .map(|record| record.entry)
        .collect())
}

fn read_entry_records(store: &PathBuf) -> Result<Vec<EntryRecord>> {
    if !store.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(store).with_context(|| format!("failed to open {}", store.display()))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("failed to read {}", store.display()))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        records.push(parse_entry_record(trimmed, index + 1)?);
    }

    Ok(records)
}

fn append_entry(store: &PathBuf, entry: &Entry) -> Result<()> {
    append_entries(store, std::slice::from_ref(entry))
}

fn append_entries(store: &PathBuf, entries: &[Entry]) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    if let Some(parent) = store.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(store)
        .with_context(|| format!("failed to open {}", store.display()))?;
    for entry in entries {
        required_event_id(entry)?;
        writeln!(file, "{}", serde_json::to_string(entry)?)
            .with_context(|| format!("failed to write {}", store.display()))?;
    }
    Ok(())
}

fn rewrite_entries(store: &PathBuf, entries: &[Entry]) -> Result<()> {
    if let Some(parent) = store.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let temp = temp_path(store);
    {
        let mut file =
            File::create(&temp).with_context(|| format!("failed to create {}", temp.display()))?;
        for entry in entries {
            required_event_id(entry)?;
            writeln!(file, "{}", serde_json::to_string(entry)?)
                .with_context(|| format!("failed to write {}", temp.display()))?;
        }
    }
    fs::rename(&temp, store).with_context(|| {
        format!(
            "failed to replace {} with {}",
            store.display(),
            temp.display()
        )
    })?;
    Ok(())
}

fn parse_entry_record(line: &str, line_number: usize) -> Result<EntryRecord> {
    let mut entry: Entry = serde_json::from_str(line)
        .with_context(|| format!("invalid JSON on line {}", line_number))?;
    let had_event_id = entry.event_id.is_some();
    if entry.event_id.is_none() {
        entry.event_id = Some(legacy_event_id(line));
    }
    Ok(EntryRecord {
        entry,
        had_event_id,
    })
}

fn required_event_id(entry: &Entry) -> Result<Uuid> {
    entry
        .event_id
        .ok_or_else(|| anyhow!("study memory entry is missing event_id"))
}

fn legacy_event_id(payload: &str) -> Uuid {
    Uuid::new_v5(&EVENT_ID_NAMESPACE, payload.as_bytes())
}

fn temp_path(path: &Path) -> PathBuf {
    let mut temp = path.to_path_buf();
    let filename = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "memory.jsonl".to_string());
    temp.set_file_name(format!("{}.tmp.{}", filename, std::process::id()));
    temp
}

async fn connect_database() -> Result<PgPool> {
    let database_url = env::var(DATABASE_URL_ENV)
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    if database_url.is_empty() {
        return Err(anyhow!(
            "{} must be set to a Postgres connection string",
            DATABASE_URL_ENV
        ));
    }

    tokio::time::timeout(
        StdDuration::from_secs(10),
        PgPoolOptions::new()
            .max_connections(3)
            .connect(&database_url),
    )
    .await
    .context("timed out connecting to the study memory database")?
    .context("failed to connect to the study memory database")
}

async fn ensure_schema(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        create table if not exists study_events (
            event_id uuid primary key,
            created_at timestamptz not null,
            received_at timestamptz not null default now(),
            device_id text not null,
            concept_id text not null,
            topic text not null,
            concept text not null,
            entry_json jsonb not null
        )
        "#,
    )
    .execute(pool)
    .await
    .context("failed to create study_events table")?;

    sqlx::query(
        "create index if not exists study_events_created_at_idx on study_events (created_at, event_id)",
    )
    .execute(pool)
    .await
    .context("failed to create study_events_created_at_idx")?;

    sqlx::query(
        "create index if not exists study_events_concept_idx on study_events (concept_id, created_at desc)",
    )
    .execute(pool)
    .await
    .context("failed to create study_events_concept_idx")?;

    sqlx::query("create index if not exists study_events_topic_idx on study_events (topic)")
        .execute(pool)
        .await
        .context("failed to create study_events_topic_idx")?;
    Ok(())
}

async fn missing_schema_columns(pool: &PgPool) -> Result<Vec<String>> {
    let columns: Vec<String> = sqlx::query_scalar(
        r#"
        select column_name
        from information_schema.columns
        where table_schema = 'public' and table_name = 'study_events'
        "#,
    )
    .fetch_all(pool)
    .await
    .context("failed to inspect study_events schema")?;
    let present: HashSet<String> = columns.into_iter().collect();
    Ok(REQUIRED_COLUMNS
        .iter()
        .filter(|column| {
            !present
                .iter()
                .any(|present_column| present_column == *column)
        })
        .map(|column| column.to_string())
        .collect())
}

async fn fetch_remote_event_ids(pool: &PgPool) -> Result<HashSet<Uuid>> {
    let ids: Vec<Uuid> = sqlx::query_scalar("select event_id from study_events")
        .fetch_all(pool)
        .await
        .context("failed to fetch remote study event IDs")?;
    Ok(ids.into_iter().collect())
}

async fn fetch_remote_events(pool: &PgPool) -> Result<Vec<(Uuid, Value)>> {
    let rows = sqlx::query(
        r#"
        select event_id, entry_json
        from study_events
        order by created_at, event_id
        "#,
    )
    .fetch_all(pool)
    .await
    .context("failed to fetch remote study events")?;

    rows.into_iter()
        .map(|row| {
            let event_id: Uuid = row.try_get("event_id")?;
            let entry_json: Value = row.try_get("entry_json")?;
            Ok((event_id, entry_json))
        })
        .collect()
}

async fn insert_remote_event(pool: &PgPool, device_id: &str, entry: &Entry) -> Result<()> {
    let event_id = required_event_id(entry)?;
    let entry_json = serde_json::to_value(entry)?;
    sqlx::query(
        r#"
        insert into study_events (
            event_id,
            created_at,
            device_id,
            concept_id,
            topic,
            concept,
            entry_json
        )
        values ($1, $2, $3, $4, $5, $6, $7)
        on conflict (event_id) do nothing
        "#,
    )
    .bind(event_id)
    .bind(entry.created_at)
    .bind(device_id)
    .bind(&entry.concept_id)
    .bind(&entry.topic)
    .bind(&entry.concept)
    .bind(entry_json)
    .execute(pool)
    .await
    .with_context(|| format!("failed to push study event {}", event_id))?;
    Ok(())
}

fn entry_from_remote_event(event_id: Uuid, entry_json: Value) -> Result<Entry> {
    let mut entry: Entry = serde_json::from_value(entry_json).with_context(|| {
        format!(
            "remote study event {} contains invalid entry JSON",
            event_id
        )
    })?;
    match entry.event_id {
        Some(payload_event_id) if payload_event_id != event_id => Err(anyhow!(
            "remote study event {} has mismatched payload event_id {}",
            event_id,
            payload_event_id
        )),
        Some(_) => Ok(entry),
        None => {
            entry.event_id = Some(event_id);
            Ok(entry)
        }
    }
}

fn sync_state_path(store: &Path) -> Result<PathBuf> {
    let parent = store
        .parent()
        .ok_or_else(|| anyhow!("store path {} has no parent directory", store.display()))?;
    Ok(parent.join("sync-state.json"))
}

fn load_sync_state(store: &Path) -> Result<SyncState> {
    let path = sync_state_path(store)?;
    if !path.exists() {
        return Ok(SyncState::default());
    }
    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&contents).with_context(|| format!("invalid JSON in {}", path.display()))
}

fn save_sync_state(store: &Path, state: &SyncState) -> Result<()> {
    let path = sync_state_path(store)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let temp = temp_path(&path);
    fs::write(&temp, format!("{}\n", serde_json::to_string_pretty(state)?))
        .with_context(|| format!("failed to write {}", temp.display()))?;
    fs::rename(&temp, &path).with_context(|| {
        format!(
            "failed to replace {} with {}",
            path.display(),
            temp.display()
        )
    })?;
    Ok(())
}

fn resolve_device_id(state: &SyncState) -> Result<String> {
    let configured = env::var(DEVICE_ID_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(device_id) = configured {
        return Ok(device_id);
    }
    if !state.device_id.trim().is_empty() {
        return Ok(state.device_id.trim().to_string());
    }
    let hostname = hostname::get()
        .context("failed to read local hostname for study sync device_id")?
        .to_string_lossy()
        .trim()
        .to_string();
    if hostname.is_empty() {
        return Err(anyhow!(
            "{} must be set because the local hostname is empty",
            DEVICE_ID_ENV
        ));
    }
    Ok(hostname)
}

fn database_config_name() -> Option<String> {
    env::var(DB_CONFIG_NAME_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env::var(DATABASE_URL_ENV)
                .ok()
                .map(|value| redact_database_url(value.trim()))
                .filter(|value| !value.is_empty())
        })
}

fn redact_database_url(database_url: &str) -> String {
    let Some((scheme, rest)) = database_url.split_once("://") else {
        return database_url.to_string();
    };
    let Some((_, host_and_database)) = rest.rsplit_once('@') else {
        return database_url.to_string();
    };
    format!("{}://<redacted>@{}", scheme, host_and_database)
}

fn latest_for_concept<'a>(entries: &'a [Entry], topic: &str, concept: &str) -> Option<&'a Entry> {
    let id = concept_id(topic, concept);
    entries
        .iter()
        .filter(|entry| entry.concept_id == id)
        .max_by_key(|entry| entry_order(entry))
}

fn latest_entries<'a, I>(entries: I) -> Vec<&'a Entry>
where
    I: IntoIterator<Item = &'a Entry>,
{
    let mut latest: HashMap<String, &'a Entry> = HashMap::new();
    for entry in entries {
        match latest.get(&entry.concept_id) {
            Some(previous) if entry_order(previous) >= entry_order(entry) => {}
            _ => {
                latest.insert(entry.concept_id.clone(), entry);
            }
        }
    }
    latest.into_values().collect()
}

fn entry_order(entry: &Entry) -> (DateTime<Utc>, Uuid) {
    (entry.created_at, entry.event_id.unwrap_or_else(Uuid::nil))
}

fn concept_id(topic: &str, concept: &str) -> String {
    format!("{}::{}", slug(topic), slug(concept))
}

fn slug(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn initial_memory(signal: Option<DifficultySignal>, confidence: Option<u8>) -> MemoryState {
    let mut difficulty = match signal {
        Some(DifficultySignal::TooEasy) => 0.25,
        Some(DifficultySignal::JustRight) | None => 0.5,
        Some(DifficultySignal::TooHard) => 0.75,
    };
    if let Some(confidence) = confidence {
        difficulty = (difficulty + (6.0 - confidence as f64) / 10.0) / 2.0;
    }
    MemoryState {
        difficulty: clamp(round2(difficulty), 0.05, 0.95),
        stability_days: 1.0,
        review_count: 0,
        lapse_count: 0,
        last_review_at: None,
        retrievability: None,
    }
}

fn update_memory(
    prior: &MemoryState,
    rating: RecallResult,
    answer_quality: f64,
    hints_used: u8,
    depth: RetrievalDepth,
    confidence: Option<u8>,
    now: DateTime<Utc>,
) -> MemoryState {
    let depth_factor = match depth {
        RetrievalDepth::Recognition => 0.85,
        RetrievalDepth::Recall => 1.0,
        RetrievalDepth::Explanation => 1.15,
        RetrievalDepth::Application => 1.35,
        RetrievalDepth::Transfer => 1.6,
    };
    let hint_penalty = 1.0 / (1.0 + hints_used as f64 * 0.25);
    let confidence_factor = confidence
        .map(|value| 0.75 + value as f64 * 0.08)
        .unwrap_or(1.0);
    let elapsed_factor = prior
        .last_review_at
        .map(|last| {
            let elapsed_days = (now - last).num_seconds().max(0) as f64 / 86_400.0;
            (1.0 + elapsed_days / prior.stability_days.max(1.0))
                .sqrt()
                .clamp(1.0, 2.0)
        })
        .unwrap_or(1.0);

    let (stability_multiplier, difficulty_delta, lapse_delta) = match rating {
        RecallResult::Again => (0.45, 0.14, 1),
        RecallResult::Hard => (1.2, 0.07, 0),
        RecallResult::Good => (2.15, -0.02, 0),
        RecallResult::Easy => (3.2, -0.08, 0),
    };

    let new_stability = match rating {
        RecallResult::Again => 1.0,
        _ => {
            prior.stability_days
                * stability_multiplier
                * depth_factor
                * hint_penalty
                * confidence_factor
                * elapsed_factor
                * (0.75 + answer_quality * 0.5)
        }
    };

    let quality_adjustment = (0.75 - answer_quality) * 0.08;
    MemoryState {
        difficulty: clamp(
            round2(prior.difficulty + difficulty_delta + quality_adjustment),
            0.05,
            0.95,
        ),
        stability_days: clamp(round2(new_stability), 1.0, 365.0),
        review_count: prior.review_count + 1,
        lapse_count: prior.lapse_count + lapse_delta,
        last_review_at: Some(now),
        retrievability: Some(round2(answer_quality)),
    }
}

fn infer_recall_result(answer_quality: f64, hints_used: u8, depth: RetrievalDepth) -> RecallResult {
    if answer_quality < 0.35 || hints_used >= 3 {
        return RecallResult::Again;
    }
    if answer_quality < 0.65 || hints_used >= 2 {
        return RecallResult::Hard;
    }
    if answer_quality >= 0.9 && hints_used == 0 && depth >= RetrievalDepth::Explanation {
        return RecallResult::Easy;
    }
    RecallResult::Good
}

fn status_from_review(rating: RecallResult, depth: RetrievalDepth, answer_quality: f64) -> Status {
    match rating {
        RecallResult::Again => Status::Introduced,
        RecallResult::Hard => Status::Practiced,
        RecallResult::Good if depth >= RetrievalDepth::Explanation && answer_quality >= 0.75 => {
            Status::Mastered
        }
        RecallResult::Easy => Status::Mastered,
        RecallResult::Good => Status::Practiced,
    }
}

fn difficulty_signal_from_rating(rating: RecallResult) -> Option<DifficultySignal> {
    Some(match rating {
        RecallResult::Again | RecallResult::Hard => DifficultySignal::TooHard,
        RecallResult::Good => DifficultySignal::JustRight,
        RecallResult::Easy => DifficultySignal::TooEasy,
    })
}

fn interval_for_status(
    status: Status,
    signal: Option<DifficultySignal>,
    confidence: Option<u8>,
) -> f64 {
    let mut days = match status {
        Status::Introduced => 1.0,
        Status::Practiced => 3.0,
        Status::Mastered => 7.0,
    };
    if matches!(signal, Some(DifficultySignal::TooHard))
        || confidence.is_some_and(|value| value <= 2)
    {
        days = 1.0;
    } else if matches!(signal, Some(DifficultySignal::TooEasy))
        && confidence.is_some_and(|value| value >= 4)
    {
        days = (days * 2.0_f64).max(7.0_f64);
    } else if status == Status::Mastered && confidence == Some(5) {
        days = 14.0;
    }
    days
}

fn due_priority(entry: &Entry) -> (DateTime<Utc>, u8, u8, String) {
    let confidence = entry.confidence.unwrap_or(99);
    let hard = if matches!(entry.difficulty_signal, Some(DifficultySignal::TooHard)) {
        0
    } else {
        1
    };
    (
        entry.next_review_at,
        hard,
        confidence,
        entry.concept_id.clone(),
    )
}

fn print_due(entry: &Entry, now: DateTime<Utc>) {
    let overdue_days = if entry.next_review_at < now {
        format!(
            ", overdue {}d",
            (now - entry.next_review_at).num_days().max(0)
        )
    } else {
        String::new()
    };
    println!(
        "- due {}{}: [{:?}] {}: {}",
        entry.next_review_at, overdue_days, entry.status, entry.topic, entry.concept
    );
    println!(
        "  memory: stability={:.2}d; difficulty={:.2}; reviews={}; lapses={}",
        entry.memory.stability_days,
        entry.memory.difficulty,
        entry.memory.review_count,
        entry.memory.lapse_count
    );
    if let Some(review) = &entry.review {
        println!(
            "  last recall: {:?}; quality={:.2}; hints={}; depth={:?}",
            review.recall_result, review.answer_quality, review.hints_used, review.retrieval_depth
        );
    }
    if let Some(next_step) = &entry.next_step {
        println!("  suggested review: {}", next_step);
    }
}

fn print_entry(entry: &Entry) {
    println!(
        "- {} [{:?}] {}: {}",
        entry.created_at, entry.status, entry.topic, entry.concept
    );
    println!(
        "  memory: due={}; stability={:.2}d; difficulty={:.2}; reviews={}; lapses={}",
        entry.next_review_at,
        entry.memory.stability_days,
        entry.memory.difficulty,
        entry.memory.review_count,
        entry.memory.lapse_count
    );
    println!("  summary: {}", entry.summary);
    if let Some(review) = &entry.review {
        println!(
            "  review: {:?}; quality={:.2}; hints={}; depth={:?}",
            review.recall_result, review.answer_quality, review.hints_used, review.retrieval_depth
        );
        print_joined("  misconceptions", &review.misconceptions, 5);
    }
    if let Some(evidence) = &entry.evidence {
        println!("  evidence: {}", evidence);
    }
    if let Some(next_step) = &entry.next_step {
        println!("  next: {}", next_step);
    }
}

fn searchable_text(entry: &Entry) -> String {
    let mut parts = vec![
        entry.topic.as_str(),
        entry.concept.as_str(),
        entry.summary.as_str(),
        entry.evidence.as_deref().unwrap_or(""),
        entry.next_step.as_deref().unwrap_or(""),
        entry.user_signal.as_deref().unwrap_or(""),
    ];
    for preference in &entry.preferences {
        parts.push(preference);
    }
    if let Some(review) = &entry.review {
        for misconception in &review.misconceptions {
            parts.push(misconception);
        }
    }
    parts.join("\n").to_lowercase()
}

fn topic_matches(entry: &Entry, topic: Option<&str>) -> bool {
    topic
        .map(|needle| entry.topic.to_lowercase().contains(&needle.to_lowercase()))
        .unwrap_or(true)
}

fn validate_confidence(confidence: Option<u8>) -> Result<()> {
    if let Some(value) = confidence {
        if !(1..=5).contains(&value) {
            return Err(anyhow!("--confidence must be between 1 and 5"));
        }
    }
    Ok(())
}

fn clean_vec(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn trim_opt(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn push_unique_opt(values: &mut Vec<String>, value: Option<&String>) {
    if let Some(value) = value {
        if !values.contains(value) {
            values.push(value.clone());
        }
    }
}

fn push_unique_all(values: &mut Vec<String>, next: &[String]) {
    for value in next {
        if !values.contains(value) {
            values.push(value.clone());
        }
    }
}

fn print_joined(label: &str, values: &[String], limit: usize) {
    if values.is_empty() {
        return;
    }
    let start = values.len().saturating_sub(limit);
    println!("{}: {}", label, values[start..].join(" | "));
}

fn duration_from_days(days: f64) -> Duration {
    Duration::seconds((days * 86_400.0).round().max(86_400.0) as i64)
}

fn days_between(start: DateTime<Utc>, end: DateTime<Utc>) -> f64 {
    round2((end - start).num_seconds().max(0) as f64 / 86_400.0)
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn infers_recall_result_from_llm_quality_signals() {
        assert_eq!(
            infer_recall_result(0.2, 0, RetrievalDepth::Recall),
            RecallResult::Again
        );
        assert_eq!(
            infer_recall_result(0.7, 2, RetrievalDepth::Application),
            RecallResult::Hard
        );
        assert_eq!(
            infer_recall_result(0.78, 0, RetrievalDepth::Recall),
            RecallResult::Good
        );
        assert_eq!(
            infer_recall_result(0.95, 0, RetrievalDepth::Explanation),
            RecallResult::Easy
        );
    }

    #[test]
    fn updates_memory_without_needing_manual_user_ratings() {
        let now = Utc.with_ymd_and_hms(2026, 5, 11, 0, 0, 0).unwrap();
        let prior = initial_memory(Some(DifficultySignal::JustRight), Some(3));
        let good = update_memory(
            &prior,
            RecallResult::Good,
            0.82,
            0,
            RetrievalDepth::Application,
            Some(4),
            now,
        );
        assert!(good.stability_days > prior.stability_days);
        assert!(good.difficulty <= prior.difficulty);
        assert_eq!(good.review_count, 1);

        let again = update_memory(
            &good,
            RecallResult::Again,
            0.1,
            3,
            RetrievalDepth::Recall,
            Some(1),
            now + Duration::days(3),
        );
        assert_eq!(again.stability_days, 1.0);
        assert!(again.difficulty > good.difficulty);
        assert_eq!(again.lapse_count, 1);
    }

    #[test]
    fn keeps_only_latest_entry_per_concept_for_due_queue() {
        let now = Utc.with_ymd_and_hms(2026, 5, 11, 0, 0, 0).unwrap();
        let old = Entry {
            event_id: Some(Uuid::new_v4()),
            created_at: now - Duration::days(5),
            algorithm_version: ALGORITHM_VERSION.to_string(),
            concept_id: concept_id("calculus", "derivative"),
            topic: "calculus".to_string(),
            concept: "derivative".to_string(),
            summary: "old".to_string(),
            status: Status::Introduced,
            memory: initial_memory(None, None),
            next_review_at: now - Duration::days(4),
            review_after_days: 1.0,
            evidence: None,
            next_step: None,
            difficulty_signal: None,
            confidence: None,
            pace: None,
            user_signal: None,
            preferences: Vec::new(),
            review: None,
        };
        let mut new = old.clone();
        new.event_id = Some(Uuid::new_v4());
        new.created_at = now;
        new.summary = "new".to_string();
        new.next_review_at = now + Duration::days(3);

        let entries = vec![new, old];
        let latest = latest_entries(entries.iter());
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].summary, "new");
    }

    #[test]
    fn assigns_stable_event_ids_to_legacy_jsonl_entries() {
        let now = Utc.with_ymd_and_hms(2026, 5, 11, 0, 0, 0).unwrap();
        let entry = Entry {
            event_id: Some(Uuid::new_v4()),
            created_at: now,
            algorithm_version: ALGORITHM_VERSION.to_string(),
            concept_id: concept_id("systems", "raft"),
            topic: "systems".to_string(),
            concept: "raft".to_string(),
            summary: "legacy".to_string(),
            status: Status::Introduced,
            memory: initial_memory(None, None),
            next_review_at: now + Duration::days(1),
            review_after_days: 1.0,
            evidence: None,
            next_step: None,
            difficulty_signal: None,
            confidence: None,
            pace: None,
            user_signal: None,
            preferences: Vec::new(),
            review: None,
        };
        let mut value = serde_json::to_value(entry).unwrap();
        value.as_object_mut().unwrap().remove("event_id");
        let line = serde_json::to_string(&value).unwrap();

        let first = parse_entry_record(&line, 1).unwrap();
        let second = parse_entry_record(&line, 1).unwrap();

        assert!(!first.had_event_id);
        assert_eq!(first.entry.event_id, second.entry.event_id);
    }
}
