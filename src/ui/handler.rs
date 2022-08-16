use crate::ui::state::{ActivePane, InputMode};
use crate::{App, TotpError, Tui};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::Backend;
use tui::widgets::Widget;

pub fn handle_key_events<B: Backend>(
    key_event: KeyEvent,
    _tui: &mut Tui<B>,
    app: &mut App,
) -> Result<(), TotpError> {
    let code = key_event.code;
    let modifiers = key_event.modifiers;

    match (code, modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => app.state.running = false,
        (KeyCode::Down, _) => app.move_down(),
        (KeyCode::Up, _) => app.move_up(),
        (KeyCode::Enter, _) => app.set_clipboard(),
        (KeyCode::Tab, _) => app.toggle_list_detail_mode(),
        _ => {}
    };
    match app.state.input_mode {
        InputMode::Normal => handle_normal_mode(key_event, app),
        InputMode::FilterList => handle_input_mode(key_event, app),
        InputMode::EditDetail => handle_edit_details(key_event, app),
    }

    Ok(())
}

pub fn handle_normal_mode(key_event: KeyEvent, app: &mut App) {
    let code = key_event.code;
    let modifiers = key_event.modifiers;
    match (code, modifiers) {
        (KeyCode::Char('/'), _) => app.state.input_mode = InputMode::FilterList,
        (KeyCode::Char('e'), _) => {
            if app.state.active_pane == ActivePane::DetailView {
                app.state.input_mode = InputMode::EditDetail;
            }
        }
        (KeyCode::Char('q'), _) => app.state.running = false,
        _ => {}
    }
}

pub fn handle_edit_details(key_event: KeyEvent, app: &mut App) {
    let code = key_event.code;
    let modifiers = key_event.modifiers;
    match (code, modifiers) {
        (KeyCode::Esc, _) => app.state.input_mode = InputMode::Normal,
        (KeyCode::Char('q'), _) => app.state.running = false,
        _ => {}
    }
}

pub fn handle_input_mode(key_event: KeyEvent, app: &mut App) {
    let code = key_event.code;
    let modifiers = key_event.modifiers;
    match (code, modifiers) {
        (KeyCode::Backspace, KeyModifiers::NONE) => {
            app.state.filter_input.pop();
            app.table_state.select(None);
        }
        (KeyCode::Char(c), KeyModifiers::NONE) => {
            app.state.filter_input.push(c);
            app.table_state.select(None);
        }
        (KeyCode::Esc, _) => {
            app.state.filter_input.clear();
            app.state.input_mode = InputMode::Normal
        }
        _ => {}
    }
}
