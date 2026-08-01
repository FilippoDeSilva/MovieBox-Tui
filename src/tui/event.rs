use crate::tui::action::Action;
use crossterm::event::{Event as CrosstermEvent, KeyEventKind};
use std::time::Duration;
use tokio::sync::mpsc;

pub struct EventHandler {
    receiver: mpsc::Receiver<Action>,
    #[allow(dead_code)]
    sender: mpsc::Sender<Action>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::channel(128);
        let event_sender = sender.clone();

        let mut tick_interval = tokio::time::interval(tick_rate);

        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            use futures::StreamExt;

            loop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        let _ = event_sender.try_send(Action::Tick);
                    }
                    Some(event) = reader.next() => {
                        match event {
                            Ok(CrosstermEvent::Key(key)) => {
                                let is_press = key.kind == KeyEventKind::Press;
                                if is_press && event_sender.send(Action::Key(key)).await.is_err() {
                                    break;
                                }
                            }
                            Ok(CrosstermEvent::Mouse(mouse)) => {
                                match mouse.kind {
                                    crossterm::event::MouseEventKind::ScrollUp => {
                                        let _ = event_sender.send(Action::Key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Up, crossterm::event::KeyModifiers::empty()))).await;
                                    }
                                    crossterm::event::MouseEventKind::ScrollDown => {
                                        let _ = event_sender.send(Action::Key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Down, crossterm::event::KeyModifiers::empty()))).await;
                                    }
                                    _ => {}
                                }
                            }
                            Ok(CrosstermEvent::FocusGained) => {
                                let _ = event_sender.send(Action::FocusChange).await;
                            }
                            Ok(CrosstermEvent::Resize(w, h)) if event_sender.send(Action::Resize(w, h)).await.is_err() => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Self { receiver, sender }
    }

    pub async fn next(&mut self) -> Option<Action> {
        self.receiver.recv().await
    }
}
