use crate::TotpError;
use crate::TotpError::UiEvent;
use crossterm::event;
use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
    /// Terminal tick.
    Tick,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender.
    sender: mpsc::Sender<Event>,
    /// Event receiver.
    receiver: mpsc::Receiver<Event>,
    /// Event handler thread.
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Result<Self, TotpError> {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);
                    if event::poll(timeout)
                        .map_err(|_e| UiEvent("Failed to fetch UI events".to_string()))
                        .unwrap_or(false)
                    {
                        match event::read()
                            .map_err(|_e| UiEvent("Failed to read UI event".to_string()))
                            .expect("Failed to read UI event")
                        {
                            CrosstermEvent::Key(e) => sender.send(Event::Key(e)),
                            CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),
                            CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                            CrosstermEvent::FocusGained
                            | CrosstermEvent::FocusLost
                            | CrosstermEvent::Paste(_) => todo!(),
                        }
                        .map_err(|_| UiEvent("Failed to send UI event".to_string()))
                        .expect("Failed to send UI event")
                    }
                    if last_tick.elapsed() > tick_rate {
                        sender
                            .send(Event::Tick)
                            .map_err(|_e| UiEvent("Failed to send tick UI event".to_string()))
                            .expect("Failed to send tick UI event");
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Ok(Self {
            sender,
            receiver,
            handler,
        })
    }

    pub fn next(&self) -> Result<Event, TotpError> {
        self.receiver.recv().map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    #[test]
    fn test_term_event() -> Result<(), TotpError> {
        let events = EventHandler::new(100)?;
        for step in 0..2 {
            if step == 1 {
                let sender = events.sender.clone();
                thread::spawn(move || {
                    sender.send(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)))
                });
            }
            match events.next()? {
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Esc {
                        assert_eq!(1, step);
                        break;
                    }
                }
                Event::Tick => assert_eq!(0, step),
                _ => {}
            };
        }
        Ok(())
    }
}
