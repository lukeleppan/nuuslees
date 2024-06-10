use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use super::{article_list::ArticleList, reader::Reader, Component};
use crate::{action::Action, config::Config, db::Database, tui::Event};

pub struct FeedView {
  feed_list: ArticleList,
  feed_reader: Reader,
}

impl FeedView {
  pub fn new(feed_list: ArticleList, feed_reader: Reader) -> Self {
    Self { feed_list, feed_reader }
  }
}

impl Component for FeedView {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.feed_list.register_action_handler(tx.clone())?;
    self.feed_reader.register_action_handler(tx)?;
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.feed_list.register_config_handler(config.clone())?;
    self.feed_reader.register_config_handler(config)?;
    Ok(())
  }

  fn init(&mut self, area: Rect) -> Result<()> {
    self.feed_list.init(area)?;
    self.feed_reader.init(area)?;
    Ok(())
  }

  fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
    if let Some(event) = event.clone() {
      self.feed_list.handle_events(Some(event.clone()))?;
      self.feed_reader.handle_events(Some(event))?;
    }
    Ok(None)
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    self.feed_list.handle_key_events(key)?;
    Ok(None)
  }

  fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
    self.feed_list.handle_mouse_events(mouse)?;
    Ok(None)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    self.feed_list.update(action.clone())?;
    self.feed_reader.update(action)?;
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    let chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
      .split(area);

    self.feed_list.draw(f, chunks[0])?;
    self.feed_reader.draw(f, chunks[1])?;
    Ok(())
  }
}
