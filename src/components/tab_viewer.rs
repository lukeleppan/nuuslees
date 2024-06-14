use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  action::Action,
  components::{
    article_list::ArticleList, feed_view::FeedView, group_view::GroupView,
    reader::Reader, Component,
  },
  config::Config,
  mode::Mode,
  tui::{Event, Frame},
};

pub struct TabViewer<'a> {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  mode: Mode,
  group_view: GroupView,
  feed_view: FeedView<'a>,
}

impl TabViewer<'_> {
  pub fn new() -> Self {
    let group_view = GroupView::new();
    let feed_list = ArticleList::new();
    let feed_reader = Reader::new();
    let feed_view = FeedView::new(feed_list, feed_reader);

    Self {
      command_tx: None,
      config: Config::default(),
      mode: Mode::GroupView,
      group_view,
      feed_view,
    }
  }
}

impl Component for TabViewer<'_> {
  fn register_action_handler(
    &mut self,
    tx: UnboundedSender<Action>,
  ) -> color_eyre::Result<()> {
    self.group_view.register_action_handler(tx.clone())?;
    self.feed_view.register_action_handler(tx.clone())?;
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(
    &mut self,
    config: Config,
  ) -> color_eyre::Result<()> {
    self.group_view.register_config_handler(config.clone())?;
    self.feed_view.register_config_handler(config.clone())?;
    self.config = config;
    Ok(())
  }

  fn init(&mut self, area: Rect) -> color_eyre::Result<()> {
    self.group_view.init(area)?;
    self.feed_view.init(area)?;
    Ok(())
  }

  fn handle_events(
    &mut self,
    event: Option<Event>,
  ) -> color_eyre::Result<Option<Action>> {
    if let Some(event) = event.clone() {
      self.group_view.handle_events(Some(event.clone()))?;
      self.feed_view.handle_events(Some(event.clone()))?;
    }
    Ok(None)
  }

  fn handle_key_events(
    &mut self,
    key: KeyEvent,
  ) -> color_eyre::Result<Option<Action>> {
    match self.mode {
      Mode::GroupView => {
        self.group_view.handle_key_events(key.clone())?;
      },
      Mode::ViewArticles(_) => {
        self.feed_view.handle_key_events(key.clone())?;
      },
      _ => {},
    }
    Ok(None)
  }

  fn handle_mouse_events(
    &mut self,
    mouse: MouseEvent,
  ) -> color_eyre::Result<Option<Action>> {
    match self.mode {
      Mode::GroupView => {
        self.group_view.handle_mouse_events(mouse.clone())?;
      },
      Mode::ViewArticles(_) => {
        self.feed_view.handle_mouse_events(mouse.clone())?;
      },
      _ => {},
    }
    Ok(None)
  }

  fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
    log::info!("Sending action: {:?}", action);
    self.group_view.update(action.clone())?;
    self.feed_view.update(action.clone())?;

    match action {
      Action::ModeChange(mode) => {
        self.mode = mode;
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> color_eyre::Result<()> {
    let main_area = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
      ])
      .split(area)[1];
    match self.mode {
      Mode::GroupView => {
        self.group_view.draw(f, main_area)?;
      },
      Mode::FeedList => {},
      Mode::ViewArticles(_) => {
        self.feed_view.draw(f, main_area)?;
      },
      _ => {},
    }
    Ok(())
  }
}
