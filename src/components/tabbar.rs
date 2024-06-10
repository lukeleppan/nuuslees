use color_eyre::eyre::Result;
use ratatui::{layout::Rect, widgets::Tabs, Frame};

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
    let tabs = Tabs::new(self.tabs.clone());
    f.render_widget(tabs, area);
    Ok(())
  }
}
