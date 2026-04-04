# Calchemy

A human-readable calendar tool with local-first storage and iCalendar export.

## Overview

Calchemy stores calendar events in a simple, human-readable flat file format inspired by Todo.txt. It handles complex recurrence rules using RFC 5545 RRULE format and exports to standard ICS format for interoperability.

## Installation

```bash
cd /home/thombruce/Development/calchemy
cargo install --path .
```

Or build without installing:
```bash
cargo build --release
./target/release/calchemy --help
```

## Format

Events are stored in a simple line-based format:

```
YYYY-MM-DD HH:MM HH:MM "Title" @location +tag +RRULE:rule +EXDATE:date
```

Example:
```
2024-01-15 09:00 10:00 Team standup @office +work +RRULE:FREQ=WEEKLY
2024-01-04 14:00 15:00 Planning meeting +RRULE:FREQ=MONTHLY;BYSETPOS=2;BYDAY=TH
2024-05-27 07:00 Bin collection +RRULE:FREQ=WEEKLY +EXDATE:2024-05-24
```

### Format Components

| Component | Description | Example |
|-----------|-------------|---------|
| Date | YYYY-MM-DD | `2024-01-15` |
| Start time | HH:MM | `09:00` |
| End time | HH:MM (optional) | `10:00` |
| Title | Event description | `"Team standup"` |
| Location | @ prefix | `@office` |
| Tags | + prefix | `+work` |
| RRULE | +RRULE: prefix | `+RRULE:FREQ=WEEKLY` |
| Exceptions | +EXDATE: prefix | `+EXDATE:2024-05-24` |

### Recurrence Rules (RRULE)

Calchemy uses RFC 5545 RRULE format. Common patterns:

| Pattern | RRULE | Description |
|---------|-------|-------------|
| Daily | `FREQ=DAILY` | Every day |
| Weekly | `FREQ=WEEKLY` | Every week |
| Monthly | `FREQ=MONTHLY` | Every month |
| Yearly | `FREQ=YEARLY` | Every year |
| Every 2nd Thursday | `FREQ=MONTHLY;BYSETPOS=2;BYDAY=TH` | 2nd Thursday of month |
| Last Friday | `FREQ=MONTHLY;BYDAY=-1FR` | Last Friday of month |
| Until date | `FREQ=WEEKLY;UNTIL=20241231` | Until specific date |

## Usage

### Default File Location

By default, Calchemy stores events at `~/.calchemy/events.cal`. You can specify a custom file with the `-f` or `--file` flag.

### Commands

#### Add an Event

```bash
# Basic event
calchemy add --date 2024-01-15 --title "Team standup"

# With time
calchemy add --date 2024-01-15 --time 09:00 --end-time 10:00 --title "Team standup"

# With recurrence
calchemy add --date 2024-01-15 --time 09:00 --end-time 10:00 --title "Team standup" --rrule "FREQ=WEEKLY"

# With location and tags
calchemy add --date 2024-01-15 --time 09:00 --title "Team standup" --location office --tag work --tag recurring

# With exception dates
calchemy add --date 2024-05-27 --time 07:00 --title "Bin collection" --rrule "FREQ=WEEKLY" --exdate 2024-05-24
```

#### List Events

```bash
# List events for current month (default)
calchemy list

# List specific month
calchemy list --month 2024-01

# List date range
calchemy list --range 2024-01-01:2024-01-31

# List all events
calchemy list --all
```

#### Export to ICS

```bash
calchemy export --output calendar.ics
```

#### Delete an Event

```bash
# First list to see indices
calchemy list --all

# Delete by index
calchemy delete 0
```

## Examples

### Weekly Standup

```bash
calchemy add \
  --date 2024-01-15 \
  --time 09:00 \
  --end-time 10:00 \
  --title "Team standup" \
  --location "Zoom" \
  --tag work \
  --rrule "FREQ=WEEKLY"
```

### Every 2nd Thursday Planning Meeting

```bash
calchemy add \
  --date 2024-01-04 \
  --time 14:00 \
  --end-time 15:00 \
  --title "Planning meeting" \
  --rrule "FREQ=MONTHLY;BYSETPOS=2;BYDAY=TH"
```

### Monthly Retro (Last Friday)

```bash
calchemy add \
  --date 2024-01-26 \
  --time 16:00 \
  --end-time 17:00 \
  --title "Retro" \
  --rrule "FREQ=MONTHLY;BYDAY=-1FR"
```

### Bin Collection with Holiday Exception

```bash
# Add weekly bin collection
calchemy add \
  --date 2024-05-27 \
  --time 07:00 \
  --title "Bin collection" \
  --rrule "FREQ=WEEKLY"

# Add exception (bank holiday - bin moved to Monday)
calchemy add \
  --date 2024-05-27 \
  --time 07:00 \
  --title "Bin collection" \
  --rrule "FREQ=WEEKLY" \
  --exdate 2024-05-24
```

## File Format Details

The Calchemy format is designed to be:
- **Human readable** - Edit directly in any text editor
- **Version control friendly** - Diff-friendly, works with git
- **Grep-able** - Easy to search with standard tools

### All-Day Events

Omit time for all-day events:
```
2024-07-04 Holiday +RRULE:FREQ=YEARLY
```

### Time Only Events

Omit end time for events without duration:
```
2024-01-15 09:00 Dentist appointment
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with custom file
cargo run -- -f /path/to/events.cal [command]
```

## License

MIT