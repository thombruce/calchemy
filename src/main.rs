use calchemy::{config::Config, Calendar, Event};
use chrono::{NaiveDate, NaiveTime};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "calchemy")]
#[command(about = "A human-readable calendar tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, global = true)]
    file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new event
    Add {
        /// Event date (YYYY-MM-DD)
        #[arg(long)]
        date: String,

        /// Start time (HH:MM)
        #[arg(long)]
        time: Option<String>,

        /// End time (HH:MM)
        #[arg(long)]
        end_time: Option<String>,

        /// End date for multi-day events (YYYY-MM-DD)
        #[arg(long)]
        end_date: Option<String>,

        /// Event title
        #[arg(long)]
        title: String,

        /// Recurrence rule (RFC 5545 format, e.g., FREQ=WEEKLY)
        #[arg(long)]
        rrule: Option<String>,

        /// Exception dates (can be specified multiple times)
        #[arg(long)]
        exdate: Vec<String>,

        /// Tags (can be specified multiple times)
        #[arg(long)]
        tag: Vec<String>,

        /// Hashtags (can be specified multiple times)
        #[arg(long)]
        hashtag: Vec<String>,

        /// Location
        #[arg(long)]
        location: Option<String>,
    },

    /// List events
    List {
        /// Month to list (YYYY-MM format)
        #[arg(long)]
        month: Option<String>,

        /// Date range (YYYY-MM-DD:YYYY-MM-DD format)
        #[arg(long)]
        range: Option<String>,

        /// Show all events (no date filtering)
        #[arg(long, short)]
        all: bool,
    },

    /// Export to ICS format
    Export {
        /// Output file path
        #[arg(long, short)]
        output: String,
    },

    /// Delete an event
    Delete {
        /// Event index (use 'calchemy list' to see indices)
        index: usize,
    },

    /// Close (complete) an event
    Close {
        /// Event index (use 'calchemy list' to see indices)
        index: usize,
    },

    /// Open (uncomplete) an event
    Open {
        /// Event index (use 'calchemy list' to see indices)
        index: usize,
    },
}

fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| format!("Invalid date format: {}", e))
}

fn parse_time(s: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(s, "%H:%M").map_err(|e| format!("Invalid time format: {}", e))
}

fn parse_month(s: &str) -> Result<(NaiveDate, NaiveDate), String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err("Month must be YYYY-MM format".to_string());
    }

    let year: i32 = parts[0].parse().map_err(|_| "Invalid year")?;
    let month: u32 = parts[1].parse().map_err(|_| "Invalid month")?;

    if !(1..=12).contains(&month) {
        return Err("Month must be between 01 and 12".to_string());
    }

    let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let end = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
    };

    Ok((start, end))
}

fn parse_range(s: &str) -> Result<(NaiveDate, NaiveDate), String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err("Range must be YYYY-MM-DD:YYYY-MM-DD format".to_string());
    }

    let start = parse_date(parts[0])?;
    let end = parse_date(parts[1])?;

    Ok((start, end))
}

fn get_calendar_path(file: &Option<PathBuf>) -> PathBuf {
    if let Some(path) = file {
        return path.clone();
    }

    if let Some(config) = Config::load() {
        return config.calendar_path();
    }

    Config::default_calendar_path()
}

fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let path = get_calendar_path(&cli.file);

    let mut calendar = if path.exists() {
        Calendar::load(path.to_str().unwrap())?
    } else {
        Calendar::new()
    };

    match &cli.command {
        Some(Commands::Add {
            date,
            time,
            end_time,
            end_date,
            title,
            rrule,
            exdate,
            tag,
            hashtag,
            location,
        }) => {
            let date = parse_date(date)?;

            let start_time = if let Some(t) = time {
                Some(parse_time(t)?)
            } else {
                None
            };

            let end_time = if let Some(t) = end_time {
                Some(parse_time(t)?)
            } else {
                None
            };

            let end_date = if let Some(d) = end_date {
                Some(parse_date(d)?)
            } else {
                None
            };

            let mut exceptions = Vec::new();
            for ex in exdate {
                exceptions.push(parse_date(ex)?);
            }

            let event = Event {
                date,
                start_time,
                end_time,
                end_date,
                title: title.clone(),
                rrule: rrule.clone(),
                every_keyword: None,
                exceptions,
                exception_keyword: None,
                tags: tag.clone(),
                hashtags: hashtag.clone(),
                location: location.clone(),
                completed: false,
            };

            calendar.add_event(event);
            calendar.save(path.to_str().unwrap())?;

            println!("Event added successfully!");
        }

        Some(Commands::List { month, range, all }) => {
            let events = if *all {
                calendar
                    .events()
                    .iter()
                    .enumerate()
                    .map(|(idx, e)| calchemy::ExpandedEvent {
                        date: e.date,
                        start_time: e.start_time,
                        end_time: e.end_time,
                        end_date: e.end_date,
                        title: e.title.clone(),
                        tags: e.tags.clone(),
                        hashtags: e.hashtags.clone(),
                        location: e.location.clone(),
                        is_exception: false,
                        completed: e.completed,
                        original_event_index: idx,
                    })
                    .collect()
            } else if let Some(m) = month {
                let (start, end) = parse_month(m)?;
                calendar.events_between(start, end)?
            } else if let Some(r) = range {
                let (start, end) = parse_range(r)?;
                calendar.events_between(start, end)?
            } else {
                let today = chrono::Local::now().date_naive();
                let end = today + chrono::Duration::days(30);
                calendar.events_between(today, end)?
            };

            println!("=== Events ===");
            for (i, event) in events.iter().enumerate() {
                if !*all && event.completed {
                    continue;
                }

                let date_str = event.date.format("%Y-%m-%d").to_string();
                let time_str = event
                    .start_time
                    .map(|t| t.format("%H:%M").to_string())
                    .unwrap_or_else(|| "all-day".to_string());
                let end_str = event
                    .end_time
                    .map(|t| format!("-{}", t.format("%H:%M")))
                    .unwrap_or_default();

                let date_range = if let Some(ref end_date) = event.end_date {
                    let end_date_str = end_date.format("%Y-%m-%d").to_string();
                    if event.start_time.is_some() {
                        let end_time_str = event
                            .end_time
                            .map(|t| t.format("%H:%M").to_string())
                            .unwrap_or_else(|| "all-day".to_string());
                        format!(
                            "{} {} - {} {}",
                            date_str, time_str, end_date_str, end_time_str
                        )
                    } else {
                        format!("{} - {}", date_str, end_date_str)
                    }
                } else if event.start_time.is_some() {
                    format!("{} {}{}", date_str, time_str, end_str)
                } else {
                    format!("{} {}", date_str, time_str)
                };

                let completed_prefix = if event.completed { "x " } else { "" };
                println!("[{}] {}{} {}", i, completed_prefix, date_range, event.title);
                if let Some(ref loc) = event.location {
                    println!("       @{}", loc);
                }
                if !event.tags.is_empty() {
                    println!("       +{}", event.tags.join(" +"));
                }
                if !event.hashtags.is_empty() {
                    println!("       #{}", event.hashtags.join(" #"));
                }
            }
        }

        Some(Commands::Export { output }) => {
            calchemy::export_ics(&calendar, output)?;
            println!("Exported to ICS successfully!");
        }

        Some(Commands::Delete { index }) => {
            if calendar.remove_event(*index).is_none() {
                return Err(format!("Invalid index: {}", index).into());
            }
            calendar.save(path.to_str().unwrap())?;

            println!("Event at index {} deleted.", index);
        }

        Some(Commands::Close { index }) => {
            let events = calendar.events_mut();
            if *index >= events.len() {
                return Err(format!("Invalid index: {}", index).into());
            }
            events[*index].completed = true;
            calendar.save(path.to_str().unwrap())?;

            println!("Event at index {} marked as complete.", index);
        }

        Some(Commands::Open { index }) => {
            let events = calendar.events_mut();
            if *index >= events.len() {
                return Err(format!("Invalid index: {}", index).into());
            }
            events[*index].completed = false;
            calendar.save(path.to_str().unwrap())?;

            println!("Event at index {} marked as incomplete.", index);
        }

        None => {
            // No command - run TUI
            run_tui()?;
        }
    }

    Ok(())
}

fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    use calchemy::tui::app::App;

    let path = get_calendar_path(&None);
    let mut app = App::new();
    app.load_calendar(path.to_str().unwrap());
    app.run()?;
    app.save_calendar();

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if any arguments were provided
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        // Run CLI mode
        run_cli()
    } else {
        // Run TUI mode
        run_tui()
    }
}
