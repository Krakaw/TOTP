use crate::ui::state::{ActivePane, EditFieldType, InputMode};
use crate::{App, TotpError, Tui};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::Backend;
use tui::style::Color;
use tui::style::Style;

use super::widgets::popup::Popup;

pub fn handle_key_events<B: Backend>(
    key_event: KeyEvent,
    _tui: &mut Tui<B>,
    app: &mut App,
) -> Result<(), TotpError> {
    let code = key_event.code;
    let modifiers = key_event.modifiers;

    match (code, modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => app.state.running = false,
        (KeyCode::Down, _) => {
            if app.state.input_mode != InputMode::EditModal {
                app.move_down();
            }
        }
        (KeyCode::Up, _) => {
            if app.state.input_mode != InputMode::EditModal {
                app.move_up();
            }
        }
        (KeyCode::Enter, _) => {
            if app.state.input_mode != InputMode::EditModal {
                app.set_clipboard();
            }
        }
        (KeyCode::Tab, _) => {
            if app.state.input_mode != InputMode::EditModal {
                app.toggle_list_detail_mode();
            }
        }
        (KeyCode::End, _) => {
            if app.state.input_mode != InputMode::EditModal {
                app.move_to_end();
            }
        }
        (KeyCode::Home, _) => {
            if app.state.input_mode != InputMode::EditModal {
                app.move_to_start();
            }
        }
        _ => {}
    };
    match app.state.input_mode {
        InputMode::Normal => handle_normal_mode(key_event, app),
        InputMode::FilterList => handle_input_mode(key_event, app),
        InputMode::EditDetail => handle_edit_details(key_event, app),
        InputMode::EditModal => handle_edit_modal(key_event, app)?,
        InputMode::DeleteConfirmation => handle_delete_confirmation(key_event, app)?,
    }

    Ok(())
}

pub fn handle_normal_mode(key_event: KeyEvent, app: &mut App) {
    let code = key_event.code;
    let modifiers = key_event.modifiers;
    match (code, modifiers) {
        (KeyCode::Char('/'), _) => app.state.input_mode = InputMode::FilterList,
        (KeyCode::Char('e'), _) => {
            match app.state.active_pane {
                ActivePane::OtpTable => {
                    // Edit account name
                    if let Some(selected) = app.table_state.selected() {
                        if let Some((_, _, _, record_id)) = app.state.display_otps.get(selected) {
                            if let Some(record) = app.state.records.iter().find(|r| r.id == *record_id) {
                                app.state.editing_record_id = Some(*record_id);
                                app.state.edit_field_type = Some(EditFieldType::AccountName);
                                app.state.edit_input = record.account.clone().unwrap_or_default();
                                app.state.input_mode = InputMode::EditModal;
                            }
                        }
                    }
                }
                ActivePane::DetailView => {
                    // Edit detail field based on selected index
                    if let Some(selected) = app.table_state.selected() {
                        if let Some((_, _, _, record_id)) = app.state.display_otps.get(selected) {
                            if let Some(record) = app.state.records.iter().find(|r| r.id == *record_id) {
                                let detail_selected = app.detail_state.selected().unwrap_or(0);
                                let (field_type, current_value) = match detail_selected {
                                    1 => (EditFieldType::Username, record.user.clone().unwrap_or_default()),
                                    2 => (EditFieldType::Note, record.note.clone().unwrap_or_default()),
                                    _ => (EditFieldType::Password, record.password.clone().unwrap_or_default()),
                                };
                                app.state.editing_record_id = Some(*record_id);
                                app.state.edit_field_type = Some(field_type);
                                app.state.edit_input = current_value;
                                app.state.input_mode = InputMode::EditModal;
                            }
                        }
                    }
                }
            }
        }
        (KeyCode::Char('d'), _) => {
            if app.state.active_pane == ActivePane::OtpTable && app.table_state.selected().is_some()
            {
                app.state.input_mode = InputMode::DeleteConfirmation;
                app.state.show_popup = Some(Popup {
                    title: "Confirm Delete".to_string(),
                    message: Some("Press 'y' to confirm or 'n' to cancel".to_string()),
                    style: Some(Style::default().fg(Color::Red)),
                    show_background: Some(true),
                    show_until: None,
                    size: None,
                    position: None,
                });
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
        (KeyCode::Char('e'), _) => app.state.running = false,
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

pub fn handle_paste(text: String, app: &mut App) -> Result<(), TotpError> {
    if app.state.input_mode == InputMode::EditModal {
        app.state.edit_input.push_str(&text);
    }
    Ok(())
}

pub fn handle_edit_modal(key_event: KeyEvent, app: &mut App) -> Result<(), TotpError> {
    let code = key_event.code;
    let modifiers = key_event.modifiers;
    match (code, modifiers) {
        (KeyCode::Backspace, KeyModifiers::NONE) => {
            app.state.edit_input.pop();
        }
        (KeyCode::Char(c), KeyModifiers::NONE) => {
            app.state.edit_input.push(c);
        }
        (KeyCode::Char(c), KeyModifiers::SHIFT) => {
            app.state.edit_input.push(c);
        }
        (KeyCode::Enter, _) => {
            // Save changes
            if let (Some(record_id), Some(field_type)) = (app.state.editing_record_id, app.state.edit_field_type.as_ref()) {
                if let Some(storage) = app.state.storage.as_mut() {
                    let mut record = storage.get_account(record_id)?;
                    match field_type {
                        EditFieldType::AccountName => {
                            record.account = Some(app.state.edit_input.clone());
                        }
                        EditFieldType::Password => {
                            record.password = Some(app.state.edit_input.clone());
                        }
                        EditFieldType::Username => {
                            record.user = Some(app.state.edit_input.clone());
                        }
                        EditFieldType::Note => {
                            record.note = Some(app.state.edit_input.clone());
                        }
                    }
                    storage.edit_account(record)?;
                    app.state.build_records()?;
                }
            }
            // Reset edit state
            app.state.input_mode = InputMode::Normal;
            app.state.edit_input.clear();
            app.state.edit_field_type = None;
            app.state.editing_record_id = None;
        }
        (KeyCode::Esc, _) => {
            // Cancel editing
            app.state.input_mode = InputMode::Normal;
            app.state.edit_input.clear();
            app.state.edit_field_type = None;
            app.state.editing_record_id = None;
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_delete_confirmation(key_event: KeyEvent, app: &mut App) -> Result<(), TotpError> {
    let code = key_event.code;
    match code {
        KeyCode::Char('y') => {
            if let Some(selected) = app.table_state.selected() {
                if let Some((_, _, _, record_id)) = app.state.display_otps.get(selected) {
                    if let Some(storage) = app.state.storage.as_mut() {
                        storage.remove_account_by_id(*record_id)?;
                        app.state.build_records()?;

                        // app.state.records = records;
                    }
                }
            }
            app.state.input_mode = InputMode::Normal;
            app.state.show_popup = None;
        }
        _ => {
            app.state.input_mode = InputMode::Normal;
            app.state.show_popup = None;
        }
    }
    Ok(())
}
