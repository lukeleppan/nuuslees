use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
  layout::{Constraint, Direction, Layout},
  prelude::Rect,
};
use reqwest::{Client, Url};
use rss::Channel;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use readability::extractor;

use crate::{
  action::Action,
  components::{
    article_list::ArticleList, feed_view::FeedView, fps::FpsCounter, home::Home, infobar::InfoBar, reader::Reader,
    tab_viewer::TabViewer, tabbar::TabBar, Component,
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
    let home = Home::new();
    let fps = FpsCounter::default();
    let tabbar = TabBar::new();
    let infobar = InfoBar::new();
    let config = Config::new()?;
    let mut db = Database::new(get_data_dir().to_str().unwrap()).await?;
    db.set_config(config.clone());
    db.init().await?;
    db.refresh_feeds().await?;
    let tab_viewer = TabViewer::new();
    let feed_list = ArticleList::new();
    let feed_reader = Reader::new();
    let feed_viewer = FeedView::new(feed_list, feed_reader);
    let mode = Mode::GroupView;
    Ok(Self {
      tick_rate,
      frame_rate,
      components: vec![Box::new(tabbar), Box::new(tab_viewer), Box::new(infobar)],
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

    let mut tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate).mouse(true);
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
    log::info!("{:?}", groups);
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
              action_tx.send(Action::Quit)?;
            }
            // if let Some(keymap) = self.config.keybindings.get(&self.mode) {
            //   if let Some(action) = keymap.get(&vec![key]) {
            //     log::info!("Got action: {action:?}");
            //     action_tx.send(action.clone())?;
            //   } else {
            //     // If the key was not handled as a single key action,
            //     // then consider it for multi-key combinations.
            //     self.last_tick_key_events.push(key);
            //
            //     // Check for multi-key combinations
            //     if let Some(action) = keymap.get(&self.last_tick_key_events) {
            //       log::info!("Got action: {action:?}");
            //       action_tx.send(action.clone())?;
            //     }
            //   }
            // };
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
              let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Fill(1), Constraint::Length(1)])
                .split(f.size());
              let tabbar = self.components.get_mut(0).unwrap();
              let r = tabbar.draw(f, layout[0]);
              if let Err(e) = r {
                action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
              }
              let feed_viewer = self.components.get_mut(1).unwrap();
              let r = feed_viewer.draw(f, layout[1]);
              if let Err(e) = r {
                action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
              }
              let infobar = self.components.get_mut(2).unwrap();
              let r = infobar.draw(f, layout[2]);
              if let Err(e) = r {
                action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap()
              }
            })?;
          },
          Action::Render => {
            tui.draw(|f| {
              let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Fill(1), Constraint::Length(1)])
                .split(f.size());
              let tabbar = self.components.get_mut(0).unwrap();
              let r = tabbar.draw(f, layout[0]);
              if let Err(e) = r {
                action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
              }
              let feed_viewer = self.components.get_mut(1).unwrap();
              let r = feed_viewer.draw(f, layout[1]);
              if let Err(e) = r {
                action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
              }
              let infobar = self.components.get_mut(2).unwrap();
              let r = infobar.draw(f, layout[2]);
              if let Err(e) = r {
                action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap()
              }
              // for component in self.components.iter_mut() {
              //   let r = component.draw(f, f.size());
              //   if let Err(e) = r {
              //     action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
              //   }
              // }
            })?;
          },
          Action::ChangeToFeedView(ref group) => {
            let feed_items = self.db.get_feed_items_from_group(group.id)?;
            self.mode = Mode::ViewArticles(feed_items);
            action_tx.send(Action::ModeChange(self.mode.clone()))?;
          },
          Action::Refresh(_) => {
            log::info!("Sending REFRESH!!!");
          },
          Action::RequestUpdateReader(ref feed_item) => {
            log::info!("Request to update reader");
            let link = feed_item.url.clone();
            let result = tokio::task::spawn_blocking(move || {
              extractor::scrape(&link)
            }).await?;

            match result {
              Ok(product) => {
                action_tx.send(Action::UpdateReader(product.content))?;
              }
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
