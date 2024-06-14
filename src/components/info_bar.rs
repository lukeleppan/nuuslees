use clap::crate_version;
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  widgets::Paragraph,
};

use crate::{components::Component, config::Config, tui::Frame};

pub struct InfoBar {
  config: Config,
}

impl InfoBar {
  pub fn new() -> Self {
    Self { config: Config::default() }
  }
}

impl Component for InfoBar {
  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> color_eyre::Result<()> {
    let info_area = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Fill(1), Constraint::Length(1)])
      .split(area)[1];

    let paragraph = Paragraph::new("Nuuslees ".to_string() + crate_version!());
    f.render_widget(paragraph, info_area);
    Ok(())
  }
}
