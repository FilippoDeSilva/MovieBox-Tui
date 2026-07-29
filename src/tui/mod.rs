pub mod action;
pub mod app;
pub mod event;
pub mod iptv_data;
pub mod state;
pub mod terminal;
pub mod theme;

pub fn clear_area(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, _theme: &theme::Theme) {
    frame.render_widget(ratatui::widgets::Clear, area);
}

pub mod screens {
    pub mod details;
    pub mod help;
    pub mod home;
    pub mod startup;
}
