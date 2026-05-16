use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

const ALGORITHM_VERSION: &str = "study-srs-v1";

#[derive(Parser, Debug)]
#[command(name = "study-mode-memory")]
#[command(about = "Persistent Study Mode memory and dependency-free spaced repetition backend.")]
struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    store: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Path,
    Record(RecordArgs),
    Review(ReviewArgs),
    Due(DueArgs),
    Show(ShowArgs),
    Search(SearchArgs),
    Profile(ProfileArgs),
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

fn main() -> Result<()> {
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
    let filtered: Vec<&Entry> = entries
        .iter()
        .filter(|entry| topic_matches(entry, args.topic.as_deref()))
        .collect();

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
    let filtered: Vec<&Entry> = entries
        .iter()
        .filter(|entry| searchable_text(entry).contains(&query))
        .collect();

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
    let filtered: Vec<&Entry> = entries
        .iter()
        .filter(|entry| topic_matches(entry, args.topic.as_deref()))
        .collect();

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

fn read_entries(store: &PathBuf) -> Result<Vec<Entry>> {
    if !store.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(store).with_context(|| format!("failed to open {}", store.display()))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("failed to read {}", store.display()))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let entry: Entry = serde_json::from_str(trimmed)
            .with_context(|| format!("invalid JSON on line {}", index + 1))?;
        entries.push(entry);
    }

    Ok(entries)
}

fn append_entry(store: &PathBuf, entry: &Entry) -> Result<()> {
    if let Some(parent) = store.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(store)
        .with_context(|| format!("failed to open {}", store.display()))?;
    writeln!(file, "{}", serde_json::to_string(entry)?)
        .with_context(|| format!("failed to write {}", store.display()))?;
    Ok(())
}

fn latest_for_concept<'a>(entries: &'a [Entry], topic: &str, concept: &str) -> Option<&'a Entry> {
    let id = concept_id(topic, concept);
    entries.iter().rev().find(|entry| entry.concept_id == id)
}

fn latest_entries<'a, I>(entries: I) -> Vec<&'a Entry>
where
    I: IntoIterator<Item = &'a Entry>,
{
    let mut latest: HashMap<String, &'a Entry> = HashMap::new();
    for entry in entries {
        latest.insert(entry.concept_id.clone(), entry);
    }
    latest.into_values().collect()
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
        new.created_at = now;
        new.summary = "new".to_string();
        new.next_review_at = now + Duration::days(3);

        let entries = vec![old, new];
        let latest = latest_entries(entries.iter());
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].summary, "new");
    }
}
