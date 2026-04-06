use chrono::NaiveDate;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use crate::Calendar;

pub struct EventList<'a> {
    calendar: &'a Calendar,
    selected_day: Option<NaiveDate>,
    selected_index: Option<usize>,
}

impl<'a> EventList<'a> {
    pub fn new(
        calendar: &'a Calendar,
        selected_day: Option<NaiveDate>,
        selected_index: Option<usize>,
    ) -> Self {
        Self {
            calendar,
            selected_day,
            selected_index,
        }
    }

    pub fn get_events_for_day(&self, day: NaiveDate) -> Vec<String> {
        let mut events = Vec::new();

        for event in self.calendar.events() {
            // Check if day falls within event's date range
            let in_range = if let Some(end_date) = event.end_date {
                event.date <= day && end_date >= day
            } else {
                event.date == day
            };

            if in_range {
                let time_str = event
                    .start_time
                    .map(|t| t.format("%H:%M").to_string())
                    .unwrap_or_else(|| "all-day".to_string());

                let end_time_str = event
                    .end_time
                    .map(|t| format!("-{}", t.format("%H:%M")))
                    .unwrap_or_default();

                let completed_mark = if event.completed { "x " } else { " " };

                // Build time strings for start and end
                let start_time_str = event.start_time.map(|t| t.format("%H:%M").to_string());
                let end_time_str = event.end_time.map(|t| t.format("%H:%M").to_string());

                // Build time display based on which times are provided
                let time_display = match (&start_time_str, &end_time_str) {
                    (Some(start), Some(end)) => format!("{}-{}", start, end),
                    (Some(start), None) => start.clone(),
                    (None, Some(end)) => format!("-{}", end),
                    (None, None) => String::new(),
                };

                // Build date/time string based on whether it's multi-day
                let date_time_str = if let Some(end_date) = event.end_date {
                    if end_date != event.date {
                        // Multi-day event
                        if !time_display.is_empty() {
                            format!(
                                "{} {} - {} {}",
                                event.date.format("%Y-%m-%d"),
                                start_time_str.unwrap(),
                                end_date.format("%Y-%m-%d"),
                                end_time_str.unwrap()
                            )
                        } else {
                            format!(
                                "{} - {}",
                                event.date.format("%Y-%m-%d"),
                                end_date.format("%Y-%m-%d")
                            )
                        }
                    } else {
                        // Single-day with end_date (same as start)
                        if !time_display.is_empty() {
                            format!("{} {}", event.date.format("%Y-%m-%d"), time_display)
                        } else {
                            event.date.format("%Y-%m-%d").to_string()
                        }
                    }
                } else {
                    // Single-day event
                    if !time_display.is_empty() {
                        format!("{} {}", event.date.format("%Y-%m-%d"), time_display)
                    } else {
                        event.date.format("%Y-%m-%d").to_string()
                    }
                };

                let event_str = format!("{} [{}] {}", completed_mark, date_time_str, event.title);
                events.push(event_str);
            }
        }

        events
    }
}

impl<'a> Widget for EventList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Events ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        block.clone().render(area, buf);

        let inner = block.inner(area);

        if let Some(day) = self.selected_day {
            let events = self.get_events_for_day(day);

            if events.is_empty() {
                let msg = "No events for this day";
                let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
                let y = inner.y + inner.height / 2;
                buf.set_string(x, y, msg, Style::default().fg(Color::DarkGray));
            } else {
                // Render with selection marker
                for (i, event_str) in events.iter().enumerate() {
                    let y = inner.y + 1 + i as u16;
                    if y >= inner.y + inner.height {
                        break;
                    }

                    let prefix = if self.selected_index == Some(i) {
                        "> "
                    } else {
                        "  "
                    };
                    let full_str = format!("{}{}", prefix, event_str);

                    let style = if event_str.starts_with("x ") {
                        Style::default().fg(Color::DarkGray)
                    } else if self.selected_index == Some(i) {
                        Style::default().fg(Color::LightBlue)
                    } else {
                        Style::default()
                    };

                    buf.set_string(inner.x, y, full_str, style);
                }
            }
        } else {
            let msg = "Select a day";
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, msg, Style::default().fg(Color::DarkGray));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    fn make_event(date: NaiveDate, title: &str) -> crate::Event {
        crate::Event {
            date,
            start_time: None,
            end_time: None,
            end_date: None,
            title: title.to_string(),
            rrule: None,
            exceptions: Vec::new(),
            tags: Vec::new(),
            hashtags: Vec::new(),
            location: None,
            completed: false,
        }
    }

    fn make_event_with_time(
        date: NaiveDate,
        start: &str,
        end: Option<&str>,
        title: &str,
    ) -> crate::Event {
        crate::Event {
            date,
            start_time: NaiveTime::parse_from_str(start, "%H:%M").ok(),
            end_time: end.and_then(|e| NaiveTime::parse_from_str(e, "%H:%M").ok()),
            end_date: None,
            title: title.to_string(),
            rrule: None,
            exceptions: Vec::new(),
            tags: Vec::new(),
            hashtags: Vec::new(),
            location: None,
            completed: false,
        }
    }

    fn make_multi_day_event(start: NaiveDate, end: NaiveDate, title: &str) -> crate::Event {
        crate::Event {
            date: start,
            start_time: None,
            end_time: None,
            end_date: Some(end),
            title: title.to_string(),
            rrule: None,
            exceptions: Vec::new(),
            tags: Vec::new(),
            hashtags: Vec::new(),
            location: None,
            completed: false,
        }
    }

    mod event_filtering {
        use super::*;

        fn make_calendar_with_events(events: Vec<crate::Event>) -> Calendar {
            let mut cal = Calendar::new();
            for event in events {
                cal.add_event(event);
            }
            cal
        }

        #[test]
        fn test_get_events_for_day_single_day() {
            let cal = make_calendar_with_events(vec![make_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Event 1",
            )]);
            let list = EventList::new(
                &cal,
                Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
                None,
            );

            let events = list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            assert_eq!(events.len(), 1);
            assert!(events[0].contains("Event 1"));
        }

        #[test]
        fn test_get_events_for_day_multi_day_returns_all_days() {
            let cal = make_calendar_with_events(vec![make_multi_day_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 18).unwrap(),
                "Multi Day",
            )]);
            let list = EventList::new(
                &cal,
                Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
                None,
            );

            let events_start =
                list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
            let events_middle =
                list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 17).unwrap());
            let events_end = list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 18).unwrap());

            assert_eq!(events_start.len(), 1);
            assert_eq!(events_middle.len(), 1);
            assert_eq!(events_end.len(), 1);
        }

        #[test]
        fn test_get_events_for_day_no_match_returns_empty() {
            let cal = make_calendar_with_events(vec![make_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Event",
            )]);
            let list = EventList::new(
                &cal,
                Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
                None,
            );

            let events = list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 20).unwrap());

            assert!(events.is_empty());
        }

        #[test]
        fn test_get_events_for_day_only_returns_events_on_that_day() {
            let cal = make_calendar_with_events(vec![
                make_event(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), "Day 15"),
                make_event(NaiveDate::from_ymd_opt(2024, 3, 16).unwrap(), "Day 16"),
                make_event(NaiveDate::from_ymd_opt(2024, 3, 17).unwrap(), "Day 17"),
            ]);
            let list = EventList::new(
                &cal,
                Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
                None,
            );

            let events = list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            assert_eq!(events.len(), 1);
            assert!(events[0].contains("Day 15"));
            assert!(!events[0].contains("Day 16"));
        }
    }

    mod event_string_formatting {
        use super::*;

        fn make_calendar_with_events(events: Vec<crate::Event>) -> Calendar {
            let mut cal = Calendar::new();
            for event in events {
                cal.add_event(event);
            }
            cal
        }

        #[test]
        fn test_format_single_day_no_times() {
            let cal = make_calendar_with_events(vec![make_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "All Day Event",
            )]);
            let list = EventList::new(
                &cal,
                Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
                None,
            );

            let events = list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            // New format: [2024-03-15] Title - no "all-day" shown
            assert!(events[0].contains("2024-03-15"));
            assert!(events[0].contains("All Day Event"));
            assert!(!events[0].contains("all-day"));
        }

        #[test]
        fn test_format_single_day_with_time() {
            let cal = make_calendar_with_events(vec![make_event_with_time(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "09:00",
                None,
                "Morning Event",
            )]);
            let list = EventList::new(
                &cal,
                Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
                None,
            );

            let events = list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            assert!(events[0].contains("09:00"));
            assert!(events[0].contains("Morning Event"));
        }

        #[test]
        fn test_format_single_day_with_time_range() {
            let cal = make_calendar_with_events(vec![make_event_with_time(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "09:00",
                Some("10:30"),
                "Meeting",
            )]);
            let list = EventList::new(
                &cal,
                Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
                None,
            );

            let events = list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            assert!(events[0].contains("09:00"));
            assert!(events[0].contains("10:30"));
            assert!(events[0].contains("Meeting"));
        }

        #[test]
        fn test_format_multi_day_event_shows_date_range() {
            let cal = make_calendar_with_events(vec![make_multi_day_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 18).unwrap(),
                "Conference",
            )]);
            let list = EventList::new(
                &cal,
                Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
                None,
            );

            let events = list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            assert!(events[0].contains("2024-03-15 - 2024-03-18"));
            assert!(events[0].contains("Conference"));
        }

        #[test]
        fn test_format_multi_day_with_times() {
            let event = crate::Event {
                date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                start_time: Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
                end_time: Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
                end_date: Some(NaiveDate::from_ymd_opt(2024, 3, 18).unwrap()),
                title: "Conference".to_string(),
                rrule: None,
                exceptions: Vec::new(),
                tags: Vec::new(),
                hashtags: Vec::new(),
                location: None,
                completed: false,
            };

            let cal = make_calendar_with_events(vec![event]);
            let list = EventList::new(
                &cal,
                Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
                None,
            );

            let events = list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            // Verify format: [2024-03-15 09:00 - 2024-03-18 17:00] Conference
            assert!(events[0].contains("2024-03-15 09:00"));
            assert!(events[0].contains("2024-03-18 17:00"));
            assert!(events[0].contains("Conference"));
        }

        #[test]
        fn test_format_completed_event_shows_x_prefix() {
            let mut event = make_event(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), "Done Task");
            event.completed = true;
            let cal = make_calendar_with_events(vec![event]);
            let list = EventList::new(
                &cal,
                Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
                None,
            );

            let events = list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            // Format: x [2024-03-15] Title
            assert!(events[0].starts_with("x "));
            assert!(events[0].contains("2024-03-15"));
            assert!(events[0].contains("Done Task"));
        }

        #[test]
        fn test_incomplete_event_shows_space_prefix() {
            let cal = make_calendar_with_events(vec![make_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Pending Task",
            )]);
            let list = EventList::new(
                &cal,
                Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
                None,
            );

            let events = list.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            // Format:   [2024-03-15] Title (starts with space)
            assert!(events[0].starts_with(" "));
            assert!(events[0].contains("2024-03-15"));
            assert!(events[0].contains("Pending Task"));
        }
    }
}
