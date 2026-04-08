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
YYYY-MM-DD HH:MM HH:MM Title @context +project key:value #hashtag
```

Example:
```
2024-01-15 09:00 10:00 Team standup @office +work rrule:FREQ=WEEKLY
2024-01-04 14:00 15:00 Planning meeting rrule:FREQ=MONTHLY;BYSETPOS=2;BYDAY=TH
2024-05-27 07:00 Bin collection rrule:FREQ=WEEKLY exdate:2024-05-24
```

### Format Components

| Component | Description | Example |
|-----------|-------------|---------|
| Date | YYYY-MM-DD | `2024-01-15` |
| Start time | HH:MM | `09:00` |
| End time | HH:MM (optional) | `10:00` |
| Title | Event description (unquoted) | `Team standup` |
| Location | @ prefix (context) | `@office` |
| Tags (projects) | + prefix | `+work` |
| Recurrence | every: or rrule: prefix | `every:week` or `rrule:FREQ=WEEKLY` |
| Exceptions | except: or exdate: prefix | `except:2024-05-24` |
| Hashtags | # prefix | `#weekly` |

### Recommended Order

For consistency (especially when using `sort` on the file), place elements in this order:

```
YYYY-MM-DD HH:MM HH:MM Title @context +project key:value #hashtag
```

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

### Human-Friendly Recurrence (every:)

For common recurrence patterns, use the `every:` keyword as a simpler alternative to RRULE:

| Pattern | every: | Description |
|---------|--------|-------------|
| Daily | `every:day` | Every day |
| Weekly | `every:week` | Every week |
| Monthly | `every:month` | Every month |
| Yearly | `every:year` | Every year |
| Weekdays | `every:weekday` | Monday to Friday |
| Weekends | `every:weekend` | Saturday and Sunday |
| Specific day | `every:monday` | Every Monday |
| Multiple days | `every:monday,wednesday` | Every Monday and Wednesday |

Examples:
```
2024-01-15 Team standup every:week
2024-01-15 Work meetings every:weekday
2024-01-15 Hiking every:weekend
2024-01-15 Yoga every:tuesday,thursday
```

### Exception Dates (except: / exdate:)

Use exceptions to skip specific occurrences of recurring events:

**`except:`** - Human-friendly format with range support:
```
2024-05-27 Bin collection every:week except:2024-05-24
2024-05-27 Bin collection every:week except:2024-03-01..2024-03-07
2024-05-27 Bin collection every:week except:2024-03-15,2024-03-22,2024-03-29
```

**`exdate:`** - RFC 5545 compliant format (comma-separated only):
```
2024-05-27 Bin collection rrule:FREQ=WEEKLY exdate:2024-05-24,2024-05-31
```

The `except:` keyword supports:
- Single dates: `except:2024-03-15`
- Comma-separated: `except:2024-03-15,2024-03-22`
- Ranges: `except:2024-03-01..2024-03-07` (expands to all dates in range)

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

### Weekly Standup with Human-Friendly Recurrence

```bash
# Using every: keyword
calchemy add \
  --date 2024-01-15 \
  --time 09:00 \
  --end-time 10:00 \
  --title "Team standup" \
  --location "Zoom" \
  --tag work

# Then edit the file to add recurrence:
# 2024-01-15 09:00 10:00 Team standup @Zoom +work every:week
```

### Vacation with Date Range Exception

```bash
# Standing desk reminder every weekday
calchemy add \
  --date 2024-01-02 \
  --title "Standing desk reminder" \
  --every weekday

# Vacation week off (range exception)
# Edit file to add:
# 2024-01-02 Standing desk reminder every:weekday except:2024-03-18..2024-03-22
```

## File Format Details

The Calchemy format is designed to be:
- **Human readable** - Edit directly in any text editor
- **Version control friendly** - Diff-friendly, works with git
- **Grep-able** - Easy to search with standard tools

### All-Day Events

Omit time for all-day events:
```
2024-07-04 Holiday rrule:FREQ=YEARLY
```

### Hashtags

Use `#` prefix for hashtags anywhere in the event line:
```
2024-01-15 09:00 Team standup #weekly #important
```

### Time Only Events

Omit end time for events without duration:
```
2024-01-15 09:00 Dentist appointment
```

## Terminal UI (TUI)

Run `calchemy` without arguments to launch the interactive terminal interface.

### Keyboard Controls

| Key | Action |
|-----|--------|
| `h` / `l` | Previous / Next month |
| `j` / `k` | Previous / Next day |
| `g` / `G` | Go to month start / end |
| `Tab` | Cycle through events on selected day |
| `a` | Add new event |
| `c` | Close/complete event |
| `o` | Open/reopen event |
| `d` | Delete event |
| `q` | Quit |
| `Esc` | Cancel input / Close dialog |

### Recurring Events in TUI

When closing or deleting a recurring event, Calchemy offers choices:

**Close Event Dialog:**
- `[1]` Close this occurrence (keep recurring)
- `[2]` Close all occurrences (mark complete)

**Delete Event Dialog:**
- `[1]` Delete this occurrence only
- `[2]` Delete this and future occurrences
- `[3]` Delete all occurrences

Press `1`, `2`, or `3` to select, or `Esc` to cancel.

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