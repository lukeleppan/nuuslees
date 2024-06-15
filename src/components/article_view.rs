use std::{ops::Index, usize};

use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use super::{article_list::ArticleList, article_reader::ArticleReader, Component};
use crate::{action::Action, config::Config, tui::Event};

pub struct ArticleView<'a> {
  idx: usize,
  selected_idx: usize,
  article_list: ArticleList,
  article_reader: ArticleReader<'a>,
}

impl<'a> ArticleView<'a> {
  pub fn new(idx: usize) -> Self {
    let article_list = ArticleList::new(idx);
    let article_reader = ArticleReader::new(idx);
    Self { idx, selected_idx: idx, article_list, article_reader }
  }
}

impl Component for ArticleView<'_> {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.article_list.register_action_handler(tx.clone())?;
    self.article_reader.register_action_handler(tx)?;
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.article_list.register_config_handler(config.clone())?;
    self.article_reader.register_config_handler(config)?;
    Ok(())
  }

  fn init(&mut self, area: Rect) -> Result<()> {
    self.article_list.init(area)?;
    self.article_reader.init(area)?;
    Ok(())
  }

  fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
    if let Some(event) = event.clone() {
      self.article_list.handle_events(Some(event.clone()))?;
      self.article_reader.handle_events(Some(event))?;
    }
    Ok(None)
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    self.article_list.handle_key_events(key)?;
    Ok(None)
  }

  fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
    self.article_list.handle_mouse_events(mouse)?;
    Ok(None)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    self.article_list.update(action.clone())?;
    self.article_reader.update(action.clone())?;

    match action {
      Action::ChangeTab(idx) => {
        self.selected_idx = idx;
      },
      Action::UpdateArticleView(idx, feed_items) => {
        if self.idx == idx {
          self.article_list.set_feed_items(feed_items);
        }
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    let chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
      .split(area);

    self.article_list.draw(f, chunks[0])?;
    self.article_reader.draw(f, chunks[1])?;
    Ok(())
  }
}
