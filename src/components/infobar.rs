use clap::crate_version;
use ratatui::{layout::Rect, widgets::Paragraph};

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
    let paragraph = Paragraph::new("Nuuslees ".to_string() + crate_version!());

    f.render_widget(paragraph, area);
    Ok(())
  }
}
