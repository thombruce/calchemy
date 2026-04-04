use calchemy::{export_ics, Calendar};
use chrono::NaiveDate;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cal = Calendar::load("examples/events.cal")?;

    println!("=== All Events ===");
    for event in cal.events() {
        println!("  {} - {}", event.date, event.title);
        if let Some(rrule) = &event.rrule {
            println!("    RRULE: {}", rrule);
        }
    }

    println!("\n=== Events in January 2024 ===");
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

    let expanded = cal.events_between(start, end)?;
    for event in &expanded {
        let time = event
            .start_time
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "all-day".to_string());
        println!("  {} {} - {}", event.date, time, event.title);
    }

    println!("\n=== Events in May 2024 (showing recurrence + exceptions) ===");
    let start = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 5, 31).unwrap();

    let expanded = cal.events_between(start, end)?;
    for event in &expanded {
        let time = event
            .start_time
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "all-day".to_string());
        println!("  {} {} - {}", event.date, time, event.title);
    }

    export_ics(&cal, "examples/calendar.ics")?;
    println!("\n=== Exported to ICS ===");

    Ok(())
}
