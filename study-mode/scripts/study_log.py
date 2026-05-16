#!/usr/bin/env python3
"""Manage the Study Mode persistent learning log."""

from __future__ import annotations

import argparse
import json
import os
import sys
from collections import Counter
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Iterable


STATUSES = ("introduced", "practiced", "mastered")
DIFFICULTIES = ("too_easy", "just_right", "too_hard")
PACES = ("slower", "steady", "faster")
DEFAULT_REVIEW_INTERVALS = {
    "introduced": 1,
    "practiced": 3,
    "mastered": 7,
}


def default_log_path() -> Path:
    base = os.environ.get("CODEX_HOME")
    root = Path(base).expanduser() if base else Path.home() / ".codex"
    return root / "study-mode" / "learning-log.jsonl"


def read_entries(path: Path) -> list[dict]:
    if not path.exists():
        return []

    entries: list[dict] = []
    with path.open("r", encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, start=1):
            stripped = line.strip()
            if not stripped:
                continue
            try:
                entry = json.loads(stripped)
            except json.JSONDecodeError as exc:
                raise SystemExit(f"Invalid JSON on line {line_number}: {exc}") from exc
            if isinstance(entry, dict):
                entries.append(entry)
    return entries


def entry_text(entry: dict) -> str:
    fields = [
        entry.get("topic", ""),
        entry.get("concept", ""),
        entry.get("summary", ""),
        entry.get("status", ""),
        entry.get("evidence", ""),
        entry.get("next_step", ""),
        entry.get("difficulty", ""),
        entry.get("confidence", ""),
        entry.get("pace", ""),
        entry.get("user_signal", ""),
        entry.get("preferences", ""),
        entry.get("next_review_at", ""),
        entry.get("review_after_days", ""),
    ]
    return "\n".join(str(field) for field in fields).lower()


def filter_entries(entries: Iterable[dict], topic: str | None = None, query: str | None = None) -> list[dict]:
    topic_lc = topic.lower() if topic else None
    query_lc = query.lower() if query else None
    results = []

    for entry in entries:
        if topic_lc and topic_lc not in str(entry.get("topic", "")).lower():
            continue
        if query_lc and query_lc not in entry_text(entry):
            continue
        results.append(entry)

    return results


def print_entries(entries: list[dict], limit: int) -> None:
    if not entries:
        print("No matching study log entries.")
        return

    for entry in entries[-limit:]:
        created = str(entry.get("created_at", "unknown"))
        topic = str(entry.get("topic", "untitled"))
        concept = str(entry.get("concept", "unspecified concept"))
        status = str(entry.get("status", "unknown"))
        summary = str(entry.get("summary", ""))
        evidence = str(entry.get("evidence", ""))
        next_step = str(entry.get("next_step", ""))
        calibration = []
        if entry.get("difficulty"):
            calibration.append(f"difficulty={entry['difficulty']}")
        if entry.get("confidence"):
            calibration.append(f"confidence={entry['confidence']}")
        if entry.get("pace"):
            calibration.append(f"pace={entry['pace']}")
        if entry.get("user_signal"):
            calibration.append(f"signal={entry['user_signal']}")
        preferences = entry.get("preferences")
        if preferences:
            joined = ", ".join(preferences) if isinstance(preferences, list) else str(preferences)
            calibration.append(f"preferences={joined}")
        if entry.get("next_review_at"):
            calibration.append(f"next_review={entry['next_review_at']}")

        print(f"- {created} [{status}] {topic}: {concept}")
        if summary:
            print(f"  summary: {summary}")
        if calibration:
            print(f"  calibration: {'; '.join(calibration)}")
        if evidence:
            print(f"  evidence: {evidence}")
        if next_step:
            print(f"  next: {next_step}")


def cmd_show(args: argparse.Namespace) -> None:
    entries = read_entries(args.log)
    print_entries(filter_entries(entries, topic=args.topic), args.limit)


def cmd_search(args: argparse.Namespace) -> None:
    entries = read_entries(args.log)
    print_entries(filter_entries(entries, query=args.query), args.limit)


def latest_values(entries: list[dict], key: str, limit: int = 5) -> list[str]:
    values = []
    for entry in reversed(entries):
        value = entry.get(key)
        if not value:
            continue
        if isinstance(value, list):
            for item in reversed(value):
                if item not in values:
                    values.append(str(item))
        elif str(value) not in values:
            values.append(str(value))
        if len(values) >= limit:
            break
    return list(reversed(values))


def parse_datetime(value: object) -> datetime | None:
    if not value:
        return None
    try:
        parsed = datetime.fromisoformat(str(value).replace("Z", "+00:00"))
    except ValueError:
        return None
    if parsed.tzinfo is None:
        return parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def int_or_none(value: object) -> int | None:
    if isinstance(value, int):
        return value
    if isinstance(value, str) and value.isdigit():
        return int(value)
    return None


def default_review_interval_days(entry: dict) -> int:
    explicit = int_or_none(entry.get("review_after_days"))
    if explicit is not None:
        return max(1, explicit)

    status = str(entry.get("status", "introduced"))
    days = DEFAULT_REVIEW_INTERVALS.get(status, 1)
    difficulty = entry.get("difficulty")
    confidence = int_or_none(entry.get("confidence"))

    if difficulty == "too_hard" or (confidence is not None and confidence <= 2):
        return 1
    if difficulty == "too_easy" and confidence is not None and confidence >= 4:
        return max(days * 2, 7)
    if status == "mastered" and confidence is not None and confidence >= 5:
        return 14
    return days


def get_next_review_at(entry: dict) -> datetime | None:
    explicit = parse_datetime(entry.get("next_review_at"))
    if explicit:
        return explicit

    created = parse_datetime(entry.get("created_at"))
    if not created:
        return None
    return created + timedelta(days=default_review_interval_days(entry))


def concept_key(entry: dict) -> tuple[str, str]:
    topic = str(entry.get("topic", "")).strip().lower()
    concept = str(entry.get("concept", "")).strip().lower()
    return topic, concept


def latest_concept_entries(entries: Iterable[dict]) -> list[dict]:
    latest: dict[tuple[str, str], dict] = {}
    for entry in entries:
        key = concept_key(entry)
        if not key[0] or not key[1]:
            continue
        latest[key] = entry
    return list(latest.values())


def print_review_entries(entries: list[dict], now: datetime, limit: int) -> None:
    if not entries:
        print("No due review items.")
        return

    for entry in entries[:limit]:
        due_at = get_next_review_at(entry)
        due_label = due_at.isoformat(timespec="seconds") if due_at else "unknown"
        overdue_days = ""
        if due_at and due_at < now:
            days = max(0, (now - due_at).days)
            overdue_days = f", overdue {days}d"
        topic = str(entry.get("topic", "untitled"))
        concept = str(entry.get("concept", "unspecified concept"))
        status = str(entry.get("status", "unknown"))
        confidence = entry.get("confidence", "?")
        difficulty = entry.get("difficulty", "?")
        next_step = str(entry.get("next_step", ""))
        print(f"- due {due_label}{overdue_days}: [{status}] {topic}: {concept}")
        print(f"  calibration: difficulty={difficulty}; confidence={confidence}")
        if next_step:
            print(f"  suggested review: {next_step}")


def cmd_due(args: argparse.Namespace) -> None:
    now = datetime.now(timezone.utc)
    entries = latest_concept_entries(filter_entries(read_entries(args.log), topic=args.topic))
    reviewable = []

    for entry in entries:
        due_at = get_next_review_at(entry)
        if not due_at:
            continue
        if args.all or due_at <= now:
            reviewable.append(entry)

    reviewable.sort(
        key=lambda entry: (
            get_next_review_at(entry) or datetime.max.replace(tzinfo=timezone.utc),
            int_or_none(entry.get("confidence")) or 99,
            0 if entry.get("difficulty") == "too_hard" else 1,
        )
    )
    print_review_entries(reviewable, now, args.limit)


def cmd_profile(args: argparse.Namespace) -> None:
    entries = filter_entries(read_entries(args.log), topic=args.topic)
    if not entries:
        print("No matching study profile yet.")
        return

    recent = entries[-args.limit :]
    status_counts = Counter(str(entry.get("status", "unknown")) for entry in recent)
    difficulty_counts = Counter(str(entry.get("difficulty")) for entry in recent if entry.get("difficulty"))
    pace_counts = Counter(str(entry.get("pace")) for entry in recent if entry.get("pace"))
    confidence_values = [
        int(entry["confidence"])
        for entry in recent
        if isinstance(entry.get("confidence"), int)
    ]
    mastered = [entry for entry in recent if entry.get("status") == "mastered"]
    next_steps = latest_values(recent, "next_step", limit=3)
    signals = latest_values(recent, "user_signal", limit=5)
    preferences = latest_values(recent, "preferences", limit=5)
    now = datetime.now(timezone.utc)
    due_entries = [
        entry
        for entry in latest_concept_entries(entries)
        if (get_next_review_at(entry) and get_next_review_at(entry) <= now)
    ]

    topic = args.topic or "all topics"
    print(f"Study profile for {topic}: {len(recent)} recent entries")
    print(f"- statuses: {dict(status_counts)}")
    if difficulty_counts:
        print(f"- difficulty: {dict(difficulty_counts)}")
    if confidence_values:
        average = sum(confidence_values) / len(confidence_values)
        print(f"- confidence: average {average:.1f}/5")
    if pace_counts:
        print(f"- pace: {dict(pace_counts)}")
    if mastered:
        concepts = [str(entry.get("concept", "unspecified concept")) for entry in mastered[-5:]]
        print(f"- recently mastered: {', '.join(concepts)}")
    if signals:
        print(f"- recent user signals: {', '.join(signals)}")
    if preferences:
        print(f"- observed preferences: {', '.join(preferences)}")
    if due_entries:
        print(f"- due reviews: {len(due_entries)}")
    if next_steps:
        print(f"- next steps: {' | '.join(next_steps)}")


def cmd_append(args: argparse.Namespace) -> None:
    args.log.parent.mkdir(parents=True, exist_ok=True)
    created_at = datetime.now(timezone.utc)
    entry = {
        "created_at": created_at.isoformat(timespec="seconds"),
        "topic": args.topic.strip(),
        "concept": args.concept.strip(),
        "summary": args.summary.strip(),
        "status": args.status,
    }
    if args.evidence:
        entry["evidence"] = args.evidence.strip()
    if args.next_step:
        entry["next_step"] = args.next_step.strip()
    if args.difficulty:
        entry["difficulty"] = args.difficulty
    if args.confidence:
        entry["confidence"] = args.confidence
    if args.pace:
        entry["pace"] = args.pace
    if args.user_signal:
        entry["user_signal"] = args.user_signal.strip()
    if args.preferences:
        entry["preferences"] = [preference.strip() for preference in args.preferences if preference.strip()]
    if args.review_after_days is not None:
        entry["review_after_days"] = args.review_after_days

    if args.next_review_at:
        next_review_at = parse_datetime(args.next_review_at)
        if not next_review_at:
            raise SystemExit(f"Invalid --next-review-at timestamp: {args.next_review_at}")
        entry["next_review_at"] = next_review_at.isoformat(timespec="seconds")
    else:
        review_days = default_review_interval_days(entry)
        entry["review_after_days"] = review_days
        entry["next_review_at"] = (created_at + timedelta(days=review_days)).isoformat(timespec="seconds")

    with args.log.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(entry, ensure_ascii=False, sort_keys=True) + "\n")

    print(f"Appended study log entry to {args.log}")


def cmd_path(args: argparse.Namespace) -> None:
    print(args.log)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Manage the Study Mode learning log.")
    parser.add_argument(
        "--log",
        type=Path,
        default=default_log_path(),
        help="Path to the JSONL learning log.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    path_parser = subparsers.add_parser("path", help="Print the default log path.")
    path_parser.set_defaults(func=cmd_path)

    show_parser = subparsers.add_parser("show", help="Show recent study log entries.")
    show_parser.add_argument("--topic", help="Filter entries by topic substring.")
    show_parser.add_argument("--limit", type=int, default=20, help="Maximum entries to print.")
    show_parser.set_defaults(func=cmd_show)

    search_parser = subparsers.add_parser("search", help="Search study log entries.")
    search_parser.add_argument("--query", required=True, help="Case-insensitive search text.")
    search_parser.add_argument("--limit", type=int, default=20, help="Maximum entries to print.")
    search_parser.set_defaults(func=cmd_search)

    due_parser = subparsers.add_parser("due", help="Show due spaced-repetition review items.")
    due_parser.add_argument("--topic", help="Filter due reviews by topic substring.")
    due_parser.add_argument("--limit", type=int, default=10, help="Maximum review items to print.")
    due_parser.add_argument("--all", action="store_true", help="Show upcoming review items too.")
    due_parser.set_defaults(func=cmd_due)

    profile_parser = subparsers.add_parser("profile", help="Summarize the user learning profile.")
    profile_parser.add_argument("--topic", help="Filter entries by topic substring.")
    profile_parser.add_argument("--limit", type=int, default=50, help="Maximum recent entries to summarize.")
    profile_parser.set_defaults(func=cmd_profile)

    append_parser = subparsers.add_parser("append", help="Append a study log entry.")
    append_parser.add_argument("--topic", required=True)
    append_parser.add_argument("--concept", required=True)
    append_parser.add_argument("--summary", required=True)
    append_parser.add_argument("--status", choices=STATUSES, required=True)
    append_parser.add_argument("--evidence")
    append_parser.add_argument("--next-step")
    append_parser.add_argument("--difficulty", choices=DIFFICULTIES)
    append_parser.add_argument("--confidence", type=int, choices=range(1, 6))
    append_parser.add_argument("--pace", choices=PACES)
    append_parser.add_argument("--user-signal")
    append_parser.add_argument("--preference", action="append", dest="preferences")
    append_parser.add_argument("--review-after-days", type=int)
    append_parser.add_argument("--next-review-at")
    append_parser.set_defaults(func=cmd_append)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)
    return 0


if __name__ == "__main__":
    sys.exit(main())
