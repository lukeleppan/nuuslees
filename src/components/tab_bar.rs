use color_eyre::eyre::Result;
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  widgets::Tabs,
  Frame,
};

use super::Component;

#[derive(Default)]
pub struct TabBar {
  pub tabs: Vec<String>,
}

impl TabBar {
  pub fn new() -> Self {
    Self { tabs: vec!["Welcome".to_string()] }
  }
}

impl Component for TabBar {
  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    let tab_area = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Length(1), Constraint::Fill(1)])
      .split(area)[0];

    let tabs = Tabs::new(self.tabs.clone());
    f.render_widget(tabs, tab_area);
    Ok(())
  }
}
