use color_eyre::eyre::Result;
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  widgets::Tabs,
  Frame,
};

use super::Component;

#[derive(Default)]
pub struct TabBar {
  tabs: Vec<String>,
  selected_tab: usize,
}

impl TabBar {
  pub fn new() -> Self {
    Self { tabs: Vec::new(), selected_tab: 0 }
  }

  pub fn add_tab(&mut self, tab: String) {
    self.tabs.push(tab);
  }

  pub fn remove_tab(&mut self, tab_idx: usize) {
    self.tabs.remove(tab_idx);
  }

  pub fn select(&mut self, tab_idx: usize) {
    self.selected_tab = tab_idx;
  }
}

impl Component for TabBar {
  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    let tabs = Tabs::new(self.tabs.clone()).select(self.selected_tab);
    f.render_widget(tabs, area);
    Ok(())
  }
}
