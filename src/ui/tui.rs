use crate::ui::app::App;
use crate::ui::event_handler::EventHandler;
use crate::ui::state::InputMode;
use crate::TotpError;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal;
use crossterm::terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use std::io;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use tui::{Frame, Terminal};

pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    pub fn init(&mut self) -> Result<(), TotpError> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        let original_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |panic| {
            disable_raw_mode().unwrap();
            crossterm::execute!(io::stdout(), LeaveAlternateScreen).unwrap();
            original_hook(panic);
        }));
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> Result<(), TotpError> {
        self.terminal
            .draw(|frame| render(app, frame))
            .map_err(|e| TotpError::Ui(e.to_string()))?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<(), TotpError> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    // let rect = frame.size();
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .margin(1)
        .split(frame.size());
    render_input(app, frame, rects[0]);
    render_otps(app, frame, rects[1]);
}

fn render_input<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>, rect: Rect) {
    let input = Paragraph::new(app.state.filter.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Filter"));
    match app.state.input_mode {
        InputMode::Input => {
            frame.set_cursor(
                // Put cursor past the end of the input text
                rect.x + app.state.filter.len() as u16 + 1,
                // Move one line down, from the border to the input line
                rect.y + 1,
            );
        }
        InputMode::Normal => {}
    }
    frame.render_widget(input, rect);
}

fn render_otps<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>, rect: Rect) {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Gray);
    let header_cells = ["Account", "OTP", "Expires In"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Blue)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let display_rows = app
        .state
        .items
        .iter()
        .filter(|(account_name, _)| {
            app.state.filter.is_empty()
                || account_name
                    .to_lowercase()
                    .contains(&app.state.filter.to_lowercase())
        })
        .map(|(account_name, generator)| {
            let (code, expiry) = generator.generate(None).unwrap();
            (account_name.to_string(), code, expiry)
        })
        .collect::<Vec<_>>();
    app.state.display_otps = display_rows.clone();
    let rows = display_rows
        .iter()
        .cloned()
        .map(|(account_name, code, expiry)| {
            let height = 1;
            let color = if expiry > 15 {
                Color::Green
            } else if expiry > 5 {
                Color::Yellow
            } else {
                Color::Red
            };
            let cells = vec![
                Cell::from(account_name),
                Cell::from(code),
                Cell::from(expiry.to_string()).style(Style::default().fg(color)),
            ];
            Row::new(cells).height(height as u16).bottom_margin(0)
        });

    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("TOTP"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(75),
            Constraint::Length(6),
            Constraint::Min(6),
        ]);
    if !app.state.items.is_empty() && app.table_state.selected().is_none() {
        app.table_state.select(Some(0));
    }
    frame.render_stateful_widget(t, rect, &mut app.table_state);
}
