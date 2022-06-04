use crate::storage::AccountName;
use crate::{Generator, Storage, Token, TotpError};
use chrono::NaiveDateTime;
use cli_clipboard::set_contents;
use crossterm::event::KeyModifiers;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::stdout;
use std::io::Write;
use std::str::FromStr;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{error::Error, io};
use tui::layout::Direction;
use tui::widgets::Paragraph;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

pub struct UiTable {
    pub storage: Storage,
    filter: String,
    state: TableState,
    items: Vec<Vec<String>>,
}

impl UiTable {
    pub fn new(storage: Storage) -> Result<(), TotpError> {
        // setup terminal
        enable_raw_mode().map_err(|e| TotpError::Ui(e.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| TotpError::Ui(e.to_string()))?;
        let backend = CrosstermBackend::new(stdout);
        let tick_rate = Duration::from_millis(250);
        let mut terminal = Terminal::new(backend).map_err(|e| TotpError::Ui(e.to_string()))?;
        let ui_table = UiTable {
            storage,
            state: TableState::default(),
            items: vec![],
            filter: String::new(),
        };
        let res = run_app(&mut terminal, ui_table, tick_rate);

        // restore terminal
        disable_raw_mode().map_err(|e| TotpError::Ui(e.to_string()))?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .map_err(|e| TotpError::Ui(e.to_string()))?;

        terminal
            .show_cursor()
            .map_err(|e| TotpError::Ui(e.to_string()))?;

        if let Err(err) = res {
            println!("{:?}", err)
        }

        Ok(())
    }

    pub fn update_items(&mut self, items: Vec<Vec<String>>) {
        self.items.clear();
        for item in items {
            self.items.push(item);
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if self.items.len() > 0 && i >= self.items.len() - 1 {
                    0
                } else {
                    usize::min(i + 1, self.items.len())
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if self.items.len() > 0 && i == 0 {
                    self.items.len() - 1
                } else {
                    usize::max(i, 1) - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn set_clipboard(&mut self) {
        if let Some(i) = self.state.selected() {
            set_contents(self.items[i][1].to_string());
        }
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: UiTable,
    tick_rate: Duration,
) -> io::Result<()> {
    let storage = app
        .storage
        .to_iter()
        .map(|(a, t)| (a.clone(), t.clone()))
        .collect::<Vec<(AccountName, Token)>>();
    let mut last_tick = Instant::now();
    loop {
        let mut items = vec![];
        for (account_name, token) in storage.clone() {
            if app.filter.is_empty()
                || account_name
                    .to_lowercase()
                    .contains(&app.filter.to_lowercase())
            {
                let generator = Generator::new(&token).unwrap();
                let (totp, expiry) = generator.generate(None).unwrap();
                let account_name = account_name.clone();
                let vec1 = vec![account_name, totp, format!("{}", expiry)];
                items.push(vec1);
            }
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                let code = key.code;
                let modifiers = key.modifiers;

                match (code, modifiers) {
                    (KeyCode::Esc, _) => app.filter.clear(),
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                    (KeyCode::Down, _) => app.next(),
                    (KeyCode::Up, _) => app.previous(),
                    (KeyCode::Enter, _) => app.set_clipboard(),
                    (KeyCode::Backspace, KeyModifiers::NONE) => {
                        app.filter.pop();
                        app.state.select(None);
                    }
                    (KeyCode::Char(c), KeyModifiers::NONE) => {
                        app.filter.push(c);
                        app.state.select(None);
                    }
                    _ => {}
                }
            }
        }
        let items_empty = items.is_empty();
        app.update_items(items);
        if !items_empty && app.state.selected().is_none() {
            app.state.select(Some(0));
        }
        terminal.draw(|f| ui(f, &mut app))?;
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut UiTable) {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .margin(1)
        .split(f.size());

    let input = Paragraph::new(app.filter.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Filter"));
    f.render_widget(input, rects[0]);
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Gray);
    let header_cells = ["Account", "OTP", "Expires In"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Blue)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().enumerate().map(|(i, c)| {
            let mut cell = Cell::from(c.clone());
            if i == 2 {
                let expiry = u64::from_str(c).unwrap_or(30);
                let color = if expiry > 15 {
                    Color::Green
                } else if expiry > 5 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                cell = cell.style(Style::default().fg(color));
            }
            cell
        });
        Row::new(cells).height(height as u16).bottom_margin(0)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("TOTP"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(45),
            Constraint::Percentage(45),
            Constraint::Percentage(10),
        ]);
    f.render_stateful_widget(t, rects[1], &mut app.state);
}
