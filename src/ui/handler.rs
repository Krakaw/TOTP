use crate::ui::state::InputMode;
use crate::{App, TotpError, Tui};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::Backend;

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
        _ => {}
    };
    match app.state.input_mode {
        InputMode::Normal => handle_normal_mode(key_event, app),
        InputMode::Input => handle_input_mode(key_event, app),
        InputMode::AddOtp => handle_add_otp_mode(key_event, app),
    }

    Ok(())
}

pub fn handle_add_otp_mode(key_event: KeyEvent, app: &mut App) {
    let code = key_event.code;
    let modifiers = key_event.modifiers;
    match (code, modifiers) {
        (KeyCode::Esc, _) => app.state.input_mode = InputMode::Normal,
        _ => {}
    }
}

pub fn handle_normal_mode(key_event: KeyEvent, app: &mut App) {
    let code = key_event.code;
    let modifiers = key_event.modifiers;
    match (code, modifiers) {
        (KeyCode::Char('/'), _) => app.state.input_mode = InputMode::Input,
        (KeyCode::Char('q'), _) => app.state.running = false,
        (KeyCode::Char('a'), _) => app.state.input_mode = InputMode::AddOtp,
        _ => {}
    }
}

pub fn handle_input_mode(key_event: KeyEvent, app: &mut App) {
    let code = key_event.code;
    let modifiers = key_event.modifiers;
    match (code, modifiers) {
        (KeyCode::Backspace, KeyModifiers::NONE) => {
            app.state.filter.pop();
            app.table_state.select(None);
        }
        (KeyCode::Char(c), KeyModifiers::NONE) => {
            app.state.filter.push(c);
            app.table_state.select(None);
        }
        (KeyCode::Esc, _) => {
            app.state.filter.clear();
            app.state.input_mode = InputMode::Normal
        }
        _ => {}
    }
}
