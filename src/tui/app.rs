use chrono::{Datelike, NaiveDate, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    Frame, Terminal,
};

use crate::{
    parse_event_line,
    tui::{calendar::CalendarView, events::EventList, input::InputDialog},
    Calendar,
};

pub struct App {
    calendar: Calendar,
    current_month: NaiveDate,
    selected_day: Option<NaiveDate>,
    selected_event_index: Option<usize>,
    show_input: bool,
    input_buffer: String,
    quit: bool,
    path: Option<String>,
    dirty: bool,
}

impl App {
    pub fn new() -> Self {
        let calendar = Calendar::default();
        let today = Utc::now().date_naive();

        Self {
            calendar,
            current_month: NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap(),
            selected_day: Some(today),
            selected_event_index: None,
            show_input: false,
            input_buffer: String::new(),
            quit: false,
            path: None,
            dirty: false,
        }
    }

    pub fn load_calendar(&mut self, path: &str) {
        self.path = Some(path.to_string());
        if std::path::Path::new(path).exists() {
            if let Ok(cal) = Calendar::load(path) {
                self.calendar = cal;
            }
        }
    }

    fn save(&self) {
        if let Some(ref path) = self.path {
            if let Err(e) = self.calendar.save(path) {
                eprintln!("Error saving: {}", e);
            }
        }
    }

    pub fn save_calendar(&mut self) {
        if !self.dirty {
            return;
        }

        if let Some(ref path) = self.path {
            if let Err(e) = self.calendar.save(path) {
                eprintln!("Error saving: {}", e);
            }
        }
        self.dirty = false;
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| self.draw(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if self.show_input {
                        self.handle_input(key.code);
                    } else {
                        self.handle_key(key.code);
                    }
                }
            }

            // Check for quit
            if !self.running() {
                break;
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn running(&self) -> bool {
        !self.quit
    }

    fn handle_key(&mut self, code: KeyCode) {
        match code {
            // Navigation - vim keys
            KeyCode::Left | KeyCode::Char('h') => {
                self.prev_month();
                self.selected_event_index = None;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.next_month();
                self.selected_event_index = None;
            }
            KeyCode::Up | KeyCode::Char('k') => self.prev_day(),
            KeyCode::Down | KeyCode::Char('j') => self.next_day(),
            KeyCode::Char('g') => self.go_to_start_of_month(),
            KeyCode::Char('G') => self.go_to_end_of_month(),

            // Event selection
            KeyCode::Tab | KeyCode::BackTab => self.cycle_event(),

            // Actions
            KeyCode::Enter => self.select_day(),
            KeyCode::Char('a') | KeyCode::Char('+') => self.show_add_dialog(),
            KeyCode::Char('x') | KeyCode::Char('c') => self.close_event(),
            KeyCode::Char('o') => self.open_event(),
            KeyCode::Char('d') => self.delete_event(),
            KeyCode::Char('q') => self.quit = true,

            // Escape to cancel
            KeyCode::Esc => {
                self.show_input = false;
                self.input_buffer.clear();
            }

            _ => {}
        }
    }

    fn handle_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Enter => self.add_event(),
            KeyCode::Esc => {
                self.show_input = false;
                self.input_buffer.clear();
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
    }

    fn prev_month(&mut self) {
        let (year, month) = if self.current_month.month() == 1 {
            (self.current_month.year() - 1, 12)
        } else {
            (self.current_month.year(), self.current_month.month() - 1)
        };
        self.current_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

        // Update selected day to stay within new month
        if let Some(day) = self.selected_day {
            let month_end = self.get_month_end();
            if day > month_end {
                self.selected_day = Some(month_end);
            }
        }

        // Reset event selection when changing month
        self.selected_event_index = None;
    }

    fn next_month(&mut self) {
        let (year, month) = if self.current_month.month() == 12 {
            (self.current_month.year() + 1, 1)
        } else {
            (self.current_month.year(), self.current_month.month() + 1)
        };
        self.current_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

        // Update selected day to stay within new month
        if let Some(day) = self.selected_day {
            if day < self.current_month {
                self.selected_day = Some(self.current_month);
            }
        }

        // Reset event selection when changing month
        self.selected_event_index = None;
    }

    fn prev_day(&mut self) {
        if let Some(day) = self.selected_day {
            if day > self.current_month {
                self.selected_day = Some(day - chrono::Duration::days(1));
                self.selected_event_index = None;
            }
        }
    }

    fn next_day(&mut self) {
        if let Some(day) = self.selected_day {
            let month_end = self.get_month_end();
            if day < month_end {
                self.selected_day = Some(day + chrono::Duration::days(1));
                self.selected_event_index = None;
            }
        }
    }

    fn go_to_start_of_month(&mut self) {
        self.selected_day = Some(self.current_month);
    }

    fn go_to_end_of_month(&mut self) {
        self.selected_day = Some(self.get_month_end());
    }

    fn get_month_end(&self) -> NaiveDate {
        let (year, month) = if self.current_month.month() == 12 {
            (self.current_month.year() + 1, 1)
        } else {
            (self.current_month.year(), self.current_month.month() + 1)
        };
        NaiveDate::from_ymd_opt(year, month, 1).unwrap() - chrono::Duration::days(1)
    }

    fn select_day(&mut self) {
        // Could open event details in the future
    }

    fn cycle_event(&mut self) {
        if let Some(day) = self.selected_day {
            let events = self.get_events_for_day(day);
            let count = events.len();
            if count > 0 {
                self.selected_event_index = match self.selected_event_index {
                    None => Some(0),
                    Some(i) => Some((i + 1) % count),
                };
            }
        }
    }

    fn get_events_for_day(&self, day: NaiveDate) -> Vec<String> {
        let mut events = Vec::new();

        for event in self.calendar.events() {
            let in_range = if let Some(end_date) = event.end_date {
                event.date <= day && end_date >= day
            } else {
                event.date == day
            };

            if in_range {
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

                let event_str = format!("{}[{}] {}", completed_mark, date_time_str, event.title);
                events.push(event_str);
            }
        }

        events
    }

    fn get_events_with_indices_for_day(&self, day: NaiveDate) -> Vec<(usize, String)> {
        let mut events = Vec::new();

        for (idx, event) in self.calendar.events().iter().enumerate() {
            let in_range = if let Some(end_date) = event.end_date {
                event.date <= day && end_date >= day
            } else {
                event.date == day
            };

            if in_range {
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

                let event_str = format!("{}[{}] {}", completed_mark, date_time_str, event.title);
                events.push((idx, event_str));
            }
        }

        events
    }

    fn show_add_dialog(&mut self) {
        self.show_input = true;
        self.input_buffer = String::new();
        if let Some(day) = self.selected_day {
            self.input_buffer = day.format("%Y-%m-%d ").to_string();
        }
    }

    fn add_event(&mut self) {
        if self.input_buffer.is_empty() {
            self.show_input = false;
            return;
        }

        // Parse the event from the input
        if let Some(event) = parse_event_line(&self.input_buffer) {
            self.calendar.add_event(event);
            self.show_input = false;
            self.input_buffer.clear();
            self.dirty = true;
            self.save();
        }
    }

    fn delete_event(&mut self) {
        if let (Some(day), Some(idx)) = (self.selected_day, self.selected_event_index) {
            let day_events = self.get_events_with_indices_for_day(day);
            if let Some((calendar_idx, _)) = day_events.get(idx) {
                self.calendar.remove_event(*calendar_idx);
                self.selected_event_index = None;
                self.dirty = true;
                self.save();
            }
        }
    }

    fn close_event(&mut self) {
        if let (Some(day), Some(idx)) = (self.selected_day, self.selected_event_index) {
            let day_events = self.get_events_with_indices_for_day(day);
            if let Some((calendar_idx, _)) = day_events.get(idx) {
                self.calendar.events_mut()[*calendar_idx].completed = true;
                self.selected_event_index = None;
                self.dirty = true;
                self.save();
            }
        }
    }

    fn open_event(&mut self) {
        if let (Some(day), Some(idx)) = (self.selected_day, self.selected_event_index) {
            let day_events = self.get_events_with_indices_for_day(day);
            if let Some((calendar_idx, _)) = day_events.get(idx) {
                self.calendar.events_mut()[*calendar_idx].completed = false;
                self.selected_event_index = None;
                self.dirty = true;
                self.save();
            }
        }
    }

    fn draw(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.size());

        // Title bar
        let title = format!("Calchemy - {}", self.current_month.format("%B %Y"));
        f.render_widget(
            ratatui::widgets::Paragraph::new(title)
                .style(Style::default().add_modifier(ratatui::style::Modifier::BOLD)),
            chunks[0],
        );

        // Main content area
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        // Calendar view
        let calendar = CalendarView::new(&self.calendar, self.current_month, self.selected_day);
        f.render_widget(calendar, main_chunks[0]);

        // Event list
        let events = EventList::new(&self.calendar, self.selected_day, self.selected_event_index);
        f.render_widget(events, main_chunks[1]);

        // Status bar
        let status = if self.show_input {
            "ADD EVENT: Press Enter to save, Esc to cancel".to_string()
        } else {
            "h/l: month  k/j: day  g/G: month start/end  Tab: cycle events  a: add  x/c: close  o: open  d: delete  q: quit".to_string()
        };
        f.render_widget(
            ratatui::widgets::Paragraph::new(status)
                .style(Style::default().fg(ratatui::style::Color::DarkGray)),
            chunks[2],
        );

        // Input dialog overlay
        if self.show_input {
            let input = InputDialog::new(&self.input_buffer);
            let area = Rect::new((f.size().width - 50) / 2, (f.size().height - 5) / 2, 50, 5);
            f.render_widget(input, area);
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn add_event_with_input(app: &mut App, input: &str) {
    if input.is_empty() {
        app.show_input = false;
        return;
    }

    if let Some(event) = parse_event_line(input) {
        app.calendar.add_event(event);
        app.show_input = false;
        app.input_buffer.clear();
        app.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    fn make_test_event(date: NaiveDate, title: &str) -> crate::Event {
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

    fn make_test_event_with_time(
        date: NaiveDate,
        start: &str,
        end: &str,
        title: &str,
    ) -> crate::Event {
        crate::Event {
            date,
            start_time: NaiveTime::parse_from_str(start, "%H:%M").ok(),
            end_time: NaiveTime::parse_from_str(end, "%H:%M").ok(),
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

    mod app_initialization {
        use super::*;

        #[test]
        fn test_app_default_initializes_correctly() {
            let app = App::new();
            assert!(app.calendar.events().is_empty());
            assert!(app.selected_day.is_some());
            assert!(app.selected_event_index.is_none());
            assert!(!app.show_input);
            assert!(app.input_buffer.is_empty());
            assert!(!app.quit);
        }

        #[test]
        fn test_app_new_sets_today_as_selected_day() {
            let app = App::new();
            let today = Utc::now().date_naive();
            assert_eq!(app.selected_day, Some(today));
        }

        #[test]
        fn test_app_new_has_no_selected_event() {
            let app = App::new();
            assert_eq!(app.selected_event_index, None);
        }

        #[test]
        fn test_app_default_today_is_current_month() {
            let app = App::new();
            let today = Utc::now().date_naive();
            assert_eq!(app.current_month.month(), today.month());
            assert_eq!(app.current_month.year(), today.year());
        }
    }

    mod calendar_loading {
        use super::*;
        use tempfile::NamedTempFile;

        #[test]
        fn test_load_calendar_from_empty_file() {
            let mut app = App::new();
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path().to_str().unwrap();

            app.load_calendar(path);

            assert!(app.calendar.events().is_empty());
        }

        #[test]
        fn test_load_calendar_parses_events_correctly() {
            let mut app = App::new();
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path().to_str().unwrap();

            std::fs::write(path, "2024-01-15 Team standup").unwrap();
            app.load_calendar(path);

            assert_eq!(app.calendar.events().len(), 1);
            assert_eq!(app.calendar.events()[0].title, "Team standup");
        }

        #[test]
        fn test_load_calendar_preserves_event_order() {
            let mut app = App::new();
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path().to_str().unwrap();

            std::fs::write(
                path,
                "2024-01-15 Event 1\n2024-01-16 Event 2\n2024-01-17 Event 3",
            )
            .unwrap();
            app.load_calendar(path);

            let events = app.calendar.events();
            assert_eq!(events.len(), 3);
            assert_eq!(events[0].title, "Event 1");
            assert_eq!(events[1].title, "Event 2");
            assert_eq!(events[2].title, "Event 3");
        }

        #[test]
        fn test_load_calendar_missing_file_creates_empty() {
            let mut app = App::new();
            app.load_calendar("/nonexistent/path/calendar.cal");
            assert!(app.calendar.events().is_empty());
        }
    }

    mod navigation_month {
        use super::*;

        #[test]
        fn test_prev_month_wraps_to_previous_year() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

            app.prev_month();

            assert_eq!(app.current_month.month(), 12);
            assert_eq!(app.current_month.year(), 2023);
        }

        #[test]
        fn test_prev_month_stays_in_same_year_if_not_january() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            app.prev_month();

            assert_eq!(app.current_month.month(), 2);
            assert_eq!(app.current_month.year(), 2024);
        }

        #[test]
        fn test_next_month_wraps_to_next_year() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 12, 15).unwrap());

            app.next_month();

            assert_eq!(app.current_month.month(), 1);
            assert_eq!(app.current_month.year(), 2025);
        }

        #[test]
        fn test_next_month_stays_in_same_year_if_not_december() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 10, 15).unwrap());

            app.next_month();

            assert_eq!(app.current_month.month(), 11);
            assert_eq!(app.current_month.year(), 2024);
        }

        #[test]
        fn test_prev_month_updates_selected_day_to_valid_date() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 31).unwrap());

            app.prev_month();

            assert!(app.selected_day.is_some());
            let day = app.selected_day.unwrap();
            assert!(day >= app.current_month);
            assert!(day <= app.get_month_end());
        }

        #[test]
        fn test_next_month_updates_selected_day_to_valid_date() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

            app.next_month();

            assert!(app.selected_day.is_some());
            let day = app.selected_day.unwrap();
            assert!(day >= app.current_month);
            assert!(day <= app.get_month_end());
        }

        #[test]
        fn test_changing_month_resets_event_selection() {
            let mut app = App::new();
            app.selected_event_index = Some(2);

            app.next_month();

            assert_eq!(app.selected_event_index, None);
        }
    }

    mod navigation_day {
        use super::*;

        #[test]
        fn test_prev_day_moves_to_previous_day() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            app.prev_day();

            assert_eq!(
                app.selected_day,
                Some(NaiveDate::from_ymd_opt(2024, 3, 14).unwrap())
            );
        }

        #[test]
        fn test_next_day_moves_to_next_day() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            app.next_day();

            assert_eq!(
                app.selected_day,
                Some(NaiveDate::from_ymd_opt(2024, 3, 16).unwrap())
            );
        }

        #[test]
        fn test_prev_day_does_not_go_before_month_start() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap());

            app.prev_day();

            assert_eq!(
                app.selected_day,
                Some(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap())
            );
        }

        #[test]
        fn test_next_day_does_not_go_past_month_end() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 31).unwrap());

            app.next_day();

            assert_eq!(
                app.selected_day,
                Some(NaiveDate::from_ymd_opt(2024, 3, 31).unwrap())
            );
        }

        #[test]
        fn test_prev_day_resets_event_selection() {
            let mut app = App::new();
            app.selected_event_index = Some(2);

            app.prev_day();

            assert_eq!(app.selected_event_index, None);
        }

        #[test]
        fn test_next_day_resets_event_selection() {
            let mut app = App::new();
            app.selected_event_index = Some(2);

            app.next_day();

            assert_eq!(app.selected_event_index, None);
        }
    }

    mod navigation_start_end_month {
        use super::*;

        #[test]
        fn test_go_to_start_of_month_sets_selected_to_first_day() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            app.go_to_start_of_month();

            assert_eq!(
                app.selected_day,
                Some(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap())
            );
        }

        #[test]
        fn test_go_to_end_of_month_sets_selected_to_last_day() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            app.go_to_end_of_month();

            assert_eq!(
                app.selected_day,
                Some(NaiveDate::from_ymd_opt(2024, 3, 31).unwrap())
            );
        }

        #[test]
        fn test_go_to_end_of_month_handles_february() {
            let mut app = App::new();
            app.current_month = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(); // 2024 is leap year
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 2, 10).unwrap());

            app.go_to_end_of_month();

            assert_eq!(
                app.selected_day,
                Some(NaiveDate::from_ymd_opt(2024, 2, 29).unwrap())
            );
        }
    }

    mod event_selection {
        use super::*;

        #[test]
        fn test_cycle_event_with_no_events_does_nothing() {
            let mut app = App::new();
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            app.cycle_event();

            assert_eq!(app.selected_event_index, None);
        }

        #[test]
        fn test_cycle_event_with_one_event_selects_it() {
            let mut app = App::new();
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Test Event",
            ));
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            app.cycle_event();

            assert_eq!(app.selected_event_index, Some(0));
        }

        #[test]
        fn test_cycle_event_with_multiple_events_cycles() {
            let mut app = App::new();
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Event 1",
            ));
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Event 2",
            ));
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Event 3",
            ));
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            assert_eq!(app.selected_event_index, None);
            app.cycle_event();
            assert_eq!(app.selected_event_index, Some(0));
            app.cycle_event();
            assert_eq!(app.selected_event_index, Some(1));
            app.cycle_event();
            assert_eq!(app.selected_event_index, Some(2));
        }

        #[test]
        fn test_cycle_event_wraps_around() {
            let mut app = App::new();
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Event 1",
            ));
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Event 2",
            ));
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            app.cycle_event();
            app.cycle_event();
            app.cycle_event();

            assert_eq!(app.selected_event_index, Some(0));
        }
    }

    mod event_operations {
        use super::*;
        use tempfile::NamedTempFile;

        #[test]
        fn test_add_event_parses_and_adds_to_calendar() {
            let mut app = App::new();

            add_event_with_input(&mut app, "2024-03-15 Team standup");

            assert_eq!(app.calendar.events().len(), 1);
            assert_eq!(app.calendar.events()[0].title, "Team standup");
        }

        #[test]
        fn test_add_event_clears_input_and_closes_dialog() {
            let mut app = App::new();
            app.show_input = true;
            app.input_buffer = "2024-03-15 Test".to_string();

            add_event_with_input(&mut app, "2024-03-15 Test");

            assert!(!app.show_input);
            assert!(app.input_buffer.is_empty());
        }

        #[test]
        fn test_add_event_with_empty_input_closes_dialog() {
            let mut app = App::new();
            app.show_input = true;
            app.input_buffer = "".to_string();

            add_event_with_input(&mut app, "");

            assert!(!app.show_input);
        }

        #[test]
        fn test_add_event_with_invalid_input_does_nothing() {
            let mut app = App::new();
            let initial_count = app.calendar.events().len();

            add_event_with_input(&mut app, "not a valid event");

            assert_eq!(app.calendar.events().len(), initial_count);
        }

        #[test]
        fn test_delete_event_removes_from_calendar() {
            let mut app = App::new();
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "To Delete",
            ));
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
            app.selected_event_index = Some(0);

            app.delete_event();

            assert!(app.calendar.events().is_empty());
        }

        #[test]
        fn test_delete_event_clears_selection() {
            let mut app = App::new();
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "To Delete",
            ));
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
            app.selected_event_index = Some(0);

            app.delete_event();

            assert_eq!(app.selected_event_index, None);
        }

        #[test]
        fn test_delete_event_with_no_selection_does_nothing() {
            let mut app = App::new();
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Event",
            ));

            app.delete_event();

            assert_eq!(app.calendar.events().len(), 1);
        }

        #[test]
        fn test_close_event_marks_as_completed() {
            let mut app = App::new();
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "To Complete",
            ));
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
            app.selected_event_index = Some(0);

            app.close_event();

            assert!(app.calendar.events()[0].completed);
        }

        #[test]
        fn test_close_event_clears_selection() {
            let mut app = App::new();
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "To Complete",
            ));
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
            app.selected_event_index = Some(0);

            app.close_event();

            assert_eq!(app.selected_event_index, None);
        }

        #[test]
        fn test_close_event_with_no_selection_does_nothing() {
            let mut app = App::new();
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Event",
            ));

            app.close_event();

            assert!(!app.calendar.events()[0].completed);
        }

        #[test]
        fn test_open_event_marks_as_incomplete() {
            let mut app = App::new();
            let mut event =
                make_test_event(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), "To Reopen");
            event.completed = true;
            app.calendar.add_event(event);
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
            app.selected_event_index = Some(0);

            app.open_event();

            assert!(!app.calendar.events()[0].completed);
        }

        #[test]
        fn test_open_event_clears_selection() {
            let mut app = App::new();
            let mut event =
                make_test_event(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), "To Reopen");
            event.completed = true;
            app.calendar.add_event(event);
            app.selected_day = Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
            app.selected_event_index = Some(0);

            app.open_event();

            assert_eq!(app.selected_event_index, None);
        }

        #[test]
        fn test_open_event_with_no_selection_does_nothing() {
            let mut app = App::new();
            let mut event = make_test_event(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), "Event");
            event.completed = true;
            app.calendar.add_event(event);

            app.open_event();

            assert!(app.calendar.events()[0].completed);
        }
    }

    mod save_functionality {
        use super::*;
        use tempfile::NamedTempFile;

        #[test]
        fn test_save_writes_to_file() {
            let mut app = App::new();
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path().to_str().unwrap();
            app.path = Some(path.to_string());

            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Test Event",
            ));

            app.save();

            let loaded = crate::Calendar::load(path).unwrap();
            assert_eq!(loaded.events().len(), 1);
            assert_eq!(loaded.events()[0].title, "Test Event");
        }

        #[test]
        fn test_save_creates_file_if_not_exists() {
            let mut app = App::new();
            let temp_dir = std::env::temp_dir();
            let path_buf = temp_dir.join(format!(
                "calchemy_test_save_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            let path = path_buf.to_str().unwrap();
            app.path = Some(path.to_string());

            app.save();

            assert!(std::path::Path::new(path).exists());

            std::fs::remove_file(path).ok();
        }

        #[test]
        fn test_save_updates_existing_file() {
            let mut app = App::new();
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path().to_str().unwrap();
            app.path = Some(path.to_string());

            // Write initial content
            std::fs::write(path, "2024-01-01 Initial event").unwrap();

            app.calendar = crate::Calendar::load(path).unwrap();
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "New Event",
            ));

            app.save();

            let loaded = crate::Calendar::load(path).unwrap();
            assert_eq!(loaded.events().len(), 2);
        }
    }

    mod event_helpers {
        use super::*;

        #[test]
        fn test_get_events_for_day_single_day_event() {
            let mut app = App::new();
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Single Day Event",
            ));

            let events = app.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            assert_eq!(events.len(), 1);
            assert!(events[0].contains("Single Day Event"));
        }

        #[test]
        fn test_get_events_for_day_multi_day_event() {
            let mut app = App::new();
            app.calendar.add_event(make_multi_day_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 18).unwrap(),
                "Multi Day Event",
            ));

            let events_start =
                app.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
            let events_middle =
                app.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 17).unwrap());
            let events_end = app.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 18).unwrap());
            let events_outside =
                app.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 14).unwrap());

            assert_eq!(events_start.len(), 1);
            assert_eq!(events_middle.len(), 1);
            assert_eq!(events_end.len(), 1);
            assert_eq!(events_outside.len(), 0);
        }

        #[test]
        fn test_get_events_for_day_no_events_returns_empty() {
            let mut app = App::new();

            let events = app.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            assert!(events.is_empty());
        }

        #[test]
        fn test_get_events_with_indices_for_day_returns_correct_indices() {
            let mut app = App::new();
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 10).unwrap(),
                "First",
            ));
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Second",
            ));
            app.calendar.add_event(make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Third",
            ));

            let events =
                app.get_events_with_indices_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            assert_eq!(events.len(), 2);
            assert_eq!(events[0].0, 1); // index in calendar
            assert_eq!(events[1].0, 2); // index in calendar
        }

        #[test]
        fn test_get_events_for_day_filters_completed() {
            let mut app = App::new();
            let mut event = make_test_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                "Completed Event",
            );
            event.completed = true;
            app.calendar.add_event(event);

            let events = app.get_events_for_day(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());

            assert_eq!(events.len(), 1);
            assert!(events[0].starts_with("x "));
        }
    }
}
