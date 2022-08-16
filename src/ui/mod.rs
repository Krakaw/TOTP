use crate::{App, Event, EventHandler, StorageTrait, TotpError, Tui};
use ::tui::backend::CrosstermBackend;
use ::tui::widgets::Widget;
use ::tui::Terminal;
use std::io;

pub mod app;
pub mod event_handler;
pub mod handler;
mod state;
pub mod tui;
pub mod widgets;

pub fn init<T: StorageTrait>(storage: T) -> Result<(), TotpError> {
    let mut app: App = App::new(storage)?;
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250)?;
    let mut tui = Tui::new(terminal, events);

    tui.init()?;
    while app.state.running {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Key(key_event) => handler::handle_key_events(key_event, &mut tui, &mut app)?,
            Event::Tick => app.tick(),
            _ => {}
        }
    }
    tui.exit()?;
    Ok(())
}
