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

    // Handle mode-specific keys first, before global handlers
    match app.state.input_mode {
        InputMode::Help => {
            handle_help(key_event, app);
            // Help mode handles all keys, don't process global handlers
            return Ok(());
        }
        InputMode::EditModal => {
            handle_edit_modal(key_event, app)?;
            // If edit modal handled the key, don't process global handlers
            if matches!(
                code,
                KeyCode::Enter | KeyCode::Esc | KeyCode::Backspace | KeyCode::Char(_)
            ) {
                return Ok(());
            }
        }
        InputMode::DeleteConfirmation => {
            handle_delete_confirmation(key_event, app)?;
            return Ok(());
        }
        _ => {}
    }

    match (code, modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => app.state.running = false,
        (KeyCode::Down, _) => {
            if app.state.input_mode != InputMode::EditModal
                && app.state.input_mode != InputMode::Help
            {
                app.move_down();
            }
        }
        (KeyCode::Up, _) => {
            if app.state.input_mode != InputMode::EditModal
                && app.state.input_mode != InputMode::Help
            {
                app.move_up();
            }
        }
        (KeyCode::Enter, _) => {
            if app.state.input_mode != InputMode::EditModal {
                app.set_clipboard();
            }
        }
        (KeyCode::Tab, _) => {
            if app.state.input_mode != InputMode::EditModal
                && app.state.input_mode != InputMode::Help
            {
                app.toggle_list_detail_mode();
            }
        }
        (KeyCode::Home, _) => {
            if app.state.input_mode != InputMode::EditModal
                && app.state.input_mode != InputMode::Help
            {
                app.move_to_start();
            }
        }
        (KeyCode::End, _) => {
            if app.state.input_mode != InputMode::EditModal
                && app.state.input_mode != InputMode::Help
            {
                app.move_to_end();
            }
        }
        (KeyCode::PageDown, _) | (KeyCode::PageUp, _) => {
            // Page up/down handled by mode-specific handlers
            if app.state.input_mode == InputMode::Help {
                // Already handled in handle_help
            }
        }
        _ => {}
    };

    match app.state.input_mode {
        InputMode::Normal => handle_normal_mode(key_event, app),
        InputMode::FilterList => handle_input_mode(key_event, app),
        InputMode::EditDetail => handle_edit_details(key_event, app),
        _ => {}
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
                            if let Some(record) =
                                app.state.records.iter().find(|r| r.id == *record_id)
                            {
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
                            if let Some(record) =
                                app.state.records.iter().find(|r| r.id == *record_id)
                            {
                                let detail_selected = app.detail_state.selected().unwrap_or(0);
                                let (field_type, current_value) = match detail_selected {
                                    1 => (
                                        EditFieldType::Username,
                                        record.user.clone().unwrap_or_default(),
                                    ),
                                    2 => (
                                        EditFieldType::Note,
                                        record.note.clone().unwrap_or_default(),
                                    ),
                                    _ => (
                                        EditFieldType::Password,
                                        record.password.clone().unwrap_or_default(),
                                    ),
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
        (KeyCode::Char('?'), _) => {
            app.state.input_mode = InputMode::Help;
            app.help_state.select(Some(0));
            app.state.show_popup = Some(Popup {
                title: "Help - Key Bindings".to_string(),
                message: Some(create_help_text()),
                style: Some(Style::default().fg(Color::Cyan)),
                show_background: Some(true),
                show_until: None,
                size: Some(super::widgets::popup::Size { x: 80, y: 35 }),
                position: Some(super::widgets::popup::Position::Center),
            });
        }
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
            if let (Some(record_id), Some(field_type)) = (
                app.state.editing_record_id,
                app.state.edit_field_type.as_ref(),
            ) {
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

fn create_help_text() -> String {
    "Global Key Bindings:\n  /              Switch to search/filter mode\n  Esc            Return to normal mode\n  Tab            Toggle between OTP table and detail view\n  Up/Down        Navigate through accounts\n  Home/End       Jump to first/last account\n  Enter          Copy OTP or selected detail to clipboard\n  Ctrl-C         Exit application\n\nNormal Mode:\n  e              Edit account name (OTP table) or detail field (detail view)\n  d              Delete selected account\n  q              Quit application\n  ?              Show this help\n\nEdit Modal:\n  Enter          Save changes\n  Esc            Cancel editing\n  Backspace      Delete character\n  Shift          Capital letters\n  Ctrl-V         Paste text\n\nHelp Mode:\n  Esc or ?       Close help\n  Up/Down        Scroll help text\n  Home/End       Jump to top/bottom".to_string()
}

pub fn handle_help(key_event: KeyEvent, app: &mut App) {
    let code = key_event.code;
    let help_lines = create_help_text().lines().count();
    // Calculate page size: help modal is 35% height, estimate ~15-20 visible lines (minus borders)
    // Use a reasonable page size that works for most terminals
    let page_size = 15;

    match code {
        KeyCode::Esc | KeyCode::Char('?') => {
            app.state.input_mode = InputMode::Normal;
            app.state.show_popup = None;
            app.help_state.select(None);
        }
        KeyCode::Down => {
            let selected = app.help_state.selected().unwrap_or(0);
            // Scroll down by page size
            let new_selected = (selected + page_size).min(help_lines.saturating_sub(1));
            app.help_state.select(Some(new_selected));
        }
        KeyCode::Up => {
            let selected = app.help_state.selected().unwrap_or(0);
            // Scroll up by page size
            if selected >= page_size {
                app.help_state.select(Some(selected - page_size));
            } else {
                app.help_state.select(Some(0));
            }
        }
        KeyCode::PageDown => {
            let selected = app.help_state.selected().unwrap_or(0);
            let new_selected = (selected + page_size).min(help_lines.saturating_sub(1));
            app.help_state.select(Some(new_selected));
        }
        KeyCode::PageUp => {
            let selected = app.help_state.selected().unwrap_or(0);
            if selected >= page_size {
                app.help_state.select(Some(selected - page_size));
            } else {
                app.help_state.select(Some(0));
            }
        }
        KeyCode::Home => {
            app.help_state.select(Some(0));
        }
        KeyCode::End => {
            app.help_state.select(Some(help_lines.saturating_sub(1)));
        }
        _ => {}
    }
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
