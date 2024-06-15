use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use tokio::sync::mpsc::UnboundedSender;

use super::{article_view, feed_view::FeedView, tab_bar::TabBar};
use crate::{
  action::Action,
  components::{
    article_list::ArticleList, article_reader::ArticleReader, article_view::ArticleView,
    group_view::GroupView, Component,
  },
  config::Config,
  mode::Mode,
  tui::{Event, Frame},
};

pub struct TabViewer {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  mode: Mode,
  tab_bar: TabBar,
  tabs: Vec<Box<dyn Component>>,
  selected_tab: usize,
}

impl TabViewer {
  pub fn new() -> Self {
    let mut tab_bar = TabBar::new();
    tab_bar.add_tab("Groups".to_string());
    let group_view = GroupView::new();

    Self {
      command_tx: None,
      config: Config::default(),
      mode: Mode::Main,
      tab_bar,
      selected_tab: 0,
      tabs: vec![Box::new(group_view)],
    }
  }

  pub fn add_new_tab(&mut self, tab_name: String, component: Box<dyn Component>) -> Result<()> {
    self.tabs.push(component);
    self.selected_tab = self.tabs.len() - 1;
    self.tab_bar.add_tab(tab_name);
    self.tab_bar.select(self.tabs.len() - 1);

    if let Some(tx) = &self.command_tx {
      tx.send(Action::ChangeTab(self.selected_tab))?;
    }

    Ok(())
  }

  pub fn remove_tab(&mut self, tab_idx: usize) {
    self.tabs.remove(tab_idx);
    self.tab_bar.remove_tab(tab_idx);

    // TODO: Update the indices of the rest of the tabs or send action to do it.
    for i in tab_idx..self.tabs.len() {}
  }

  pub fn select_tab(&mut self, idx: usize) -> Result<()> {
    self.selected_tab = idx;
    self.tab_bar.select(idx);

    if let Some(tx) = &self.command_tx {
      tx.send(Action::ChangeTab(idx))?;
    }

    Ok(())
  }
}

impl Component for TabViewer {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
    for component in &mut self.tabs {
      component.register_action_handler(tx.clone())?;
    }
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
    for component in &mut self.tabs {
      component.register_config_handler(config.clone())?;
    }
    self.config = config;
    Ok(())
  }

  fn init(&mut self, area: Rect) -> color_eyre::Result<()> {
    for component in &mut self.tabs {
      component.init(area)?;
    }
    Ok(())
  }

  fn handle_events(&mut self, event: Option<Event>) -> color_eyre::Result<Option<Action>> {
    if let Some(event) = event.clone() {
      for component in &mut self.tabs {
        component.handle_events(Some(event.clone()))?;
      }
    }

    let r = match event {
      Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
      Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
      _ => None,
    };
    Ok(r)
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
    // for component in &mut self.tabs {
    //   component.handle_key_events(key.clone())?;
    // }

    if key.modifiers.contains(KeyModifiers::SHIFT) {
      match key.code {
        KeyCode::Char('H') => {
          if self.selected_tab == 0 {
            self.select_tab(self.tabs.len() - 1)?;
          } else {
            self.select_tab(self.selected_tab - 1)?;
          }
        },
        KeyCode::Char('L') => {
          self.select_tab((self.selected_tab + 1) % self.tabs.len())?;
        },
        _ => {},
      };
    } else {
      match key.code {
        KeyCode::Char('x') => {
          if self.selected_tab != 0 {
            self.remove_tab(self.selected_tab);
            self.select_tab(self.selected_tab - 1)?;
            return Ok(Some(Action::RemoveTab(self.selected_tab + 1)));
          }
        },
        _ => {},
      }
    }

    Ok(None)
  }

  fn handle_mouse_events(&mut self, mouse: MouseEvent) -> color_eyre::Result<Option<Action>> {
    for component in &mut self.tabs {
      component.handle_mouse_events(mouse.clone())?;
    }
    Ok(None)
  }

  fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
    for component in &mut self.tabs {
      component.update(action.clone())?;
    }

    match action {
      Action::ModeChange(mode) => {
        self.mode = mode;
      },
      Action::NewTabFeedView(group) => {
        let mut feed_view = FeedView::new(self.tabs.len(), group.clone());
        if let Some(tx) = &self.command_tx {
          feed_view.register_action_handler(tx.clone())?;
        }
        self.add_new_tab(group.name.clone(), Box::new(feed_view))?;
        return Ok(Some(Action::RequestUpdateFeedView(self.tabs.len() - 1, group)));
      },
      Action::NewTabArticleViewAll => {
        let mut article_view = ArticleView::new(self.tabs.len());
        if let Some(tx) = &self.command_tx {
          article_view.register_action_handler(tx.clone())?;
        }
        self.add_new_tab("All Articles".to_string(), Box::new(article_view))?;
        return Ok(Some(Action::RequestUpdateArticleViewAll(self.tabs.len() - 1)));
      },
      Action::NewTabArticleViewFeed(feed) => {
        let mut article_view = ArticleView::new(self.tabs.len());
        if let Some(tx) = &self.command_tx {
          article_view.register_action_handler(tx.clone())?;
        }
        self.add_new_tab(feed.name.clone(), Box::new(article_view))?;
        log::info!("Sending RequestUpdateArticleViewFeed");
        return Ok(Some(Action::RequestUpdateArticleViewFeed(self.tabs.len() - 1, feed)));
      },
      Action::NewTabArticleViewGroup(group) => {
        let mut article_view = ArticleView::new(self.tabs.len());
        if let Some(tx) = &self.command_tx {
          article_view.register_action_handler(tx.clone())?;
        }
        self.add_new_tab(group.name.clone(), Box::new(article_view))?;
        return Ok(Some(Action::RequestUpdateArticleViewGroup(self.tabs.len(), group)));
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> color_eyre::Result<()> {
    let layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Length(1), Constraint::Fill(1), Constraint::Length(1)])
      .split(area);

    let tab_area = layout[0];
    self.tab_bar.draw(f, tab_area)?;

    let main_area = layout[1];
    if let Some(component) = self.tabs.get_mut(self.selected_tab) {
      component.draw(f, main_area)?;
    }

    Ok(())
  }
}
