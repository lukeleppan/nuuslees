use std::ops::Index;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
  layout::{Constraint, Direction, Layout},
  prelude::Rect,
};
use readability::extractor;
use reqwest::{Client, Url};
use rss::Channel;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
  action::Action,
  components::{
    article_list::ArticleList, article_reader::ArticleReader, article_view::ArticleView,
    info_bar::InfoBar, popup_quit::QuitPopup, tab_bar::TabBar, tab_viewer::TabViewer, Component,
  },
  config::Config,
  db::{Database, DbError},
  mode::Mode,
  tui,
  utils::get_data_dir,
};

pub struct App {
  pub config: Config,
  pub db: Database,
  pub tick_rate: f64,
  pub frame_rate: f64,
  pub components: Vec<Box<dyn Component>>,
  pub should_quit: bool,
  pub should_suspend: bool,
  pub mode: Mode,
  pub last_tick_key_events: Vec<KeyEvent>,
  pub feeds: Option<Vec<Channel>>,
}

impl App {
  pub async fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
    let config = Config::new()?;
    let mut db = Database::new(get_data_dir().to_str().unwrap()).await?;
    db.set_config(config.clone());
    db.init().await?;
    db.refresh_feeds().await?;
    let tabbar = TabBar::new();
    let infobar = InfoBar::new();
    let tab_viewer = TabViewer::new();
    let quit_popup = QuitPopup::new();
    let mode = Mode::Main;
    Ok(Self {
      tick_rate,
      frame_rate,
      components: vec![Box::new(tab_viewer), Box::new(infobar), Box::new(quit_popup)],
      should_quit: false,
      should_suspend: false,
      config,
      db,
      mode,
      last_tick_key_events: Vec::new(),
      feeds: None,
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    let mut tui =
      tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate).mouse(true);
    tui.enter()?;

    for component in self.components.iter_mut() {
      component.register_action_handler(action_tx.clone())?;
    }

    for component in self.components.iter_mut() {
      component.register_config_handler(self.config.clone())?;
    }

    for component in self.components.iter_mut() {
      component.init(tui.size()?)?;
    }

    let groups = self.db.get_groups()?;
    action_tx.send(Action::Refresh(groups))?;

    loop {
      if let Some(e) = tui.next().await {
        match e {
          tui::Event::Quit => action_tx.send(Action::Quit)?,
          tui::Event::Tick => action_tx.send(Action::Tick)?,
          tui::Event::Render => action_tx.send(Action::Render)?,
          tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
          tui::Event::Key(key) => {
            if key.code == crossterm::event::KeyCode::Char('q') {
              action_tx.send(Action::ConfirmQuit)?;
            }
          },

          _ => {},
        }
        for component in self.components.iter_mut() {
          if let Some(action) = component.handle_events(Some(e.clone()))? {
            action_tx.send(action)?;
          }
        }
      }

      while let Ok(action) = action_rx.try_recv() {
        if action != Action::Tick && action != Action::Render {
          log::debug!("{action:?}");
        }
        match action {
          Action::Tick => {
            self.last_tick_key_events.drain(..);
          },
          Action::Quit => self.should_quit = true,
          Action::Suspend => self.should_suspend = true,
          Action::Resume => self.should_suspend = false,
          Action::Resize(w, h) => {
            tui.resize(Rect::new(0, 0, w, h))?;
            tui.draw(|f| {
              for component in self.components.iter_mut() {
                let r = component.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
              }
            })?;
          },
          Action::Render => {
            tui.draw(|f| {
              for component in self.components.iter_mut() {
                let r = component.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
              }
            })?;
          },
          Action::RequestUpdateFeedView(idx, ref group) => {
            let feeds = self.db.get_feeds_from_group(group.id)?;
            action_tx.send(Action::UpdateFeedView(idx, feeds))?;
          },
          Action::RequestUpdateArticleViewAll(idx) => {
            let feed_items = self.db.get_feed_items()?;
            action_tx.send(Action::UpdateArticleView(idx, feed_items))?;
          },
          Action::RequestUpdateArticleViewFeed(idx, ref feed) => {
            let feed_items = self.db.get_feed_items_from_feed(feed.id)?;
            log::info!("Sending UpdateArticleViewFeed");
            action_tx.send(Action::UpdateArticleView(idx, feed_items))?;
          },
          Action::RequestUpdateArticleViewGroup(idx, ref group) => {
            let feed_items = self.db.get_feed_items_from_group(group.id)?;
            action_tx.send(Action::UpdateArticleView(idx, feed_items))?;
          },
          Action::Refresh(_) => {},
          Action::RequestUpdateReader(idx, ref feed_item) => {
            let link = feed_item.url.clone();
            let result = tokio::task::spawn_blocking(move || extractor::scrape(&link)).await?;

            match result {
              Ok(product) => {
                action_tx.send(Action::UpdateReader(idx, product.content))?;
              },
              Err(_) => log::error!("Failed to display post."),
            }
          },
          _ => {},
        }
        for component in self.components.iter_mut() {
          if let Some(action) = component.update(action.clone())? {
            action_tx.send(action)?
          };
        }
      }
      if self.should_suspend {
        tui.suspend()?;
        action_tx.send(Action::Resume)?;
        tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()?;
      } else if self.should_quit {
        tui.stop()?;
        break;
      }
    }
    tui.exit()?;
    Ok(())
  }
}
