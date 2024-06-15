use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, MouseEvent};
use ratatui::{
  layout::Rect,
  style::{Color, Modifier, Style},
  text::{Line, Text},
  widgets::{Block, BorderType, List, ListItem, ListState},
};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
  action::Action,
  config::Config,
  db::{Feed, Group},
  mode::Mode,
  tui::Frame,
};

pub struct FeedView {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  mode: Mode,
  group: Group,
  idx: usize,
  selected_idx: usize,
  feeds: Vec<Feed>,
  state: ListState,
}

impl FeedView {
  pub fn new(idx: usize, group: Group) -> Self {
    Self {
      command_tx: None,
      config: Config::default(),
      mode: Mode::default(),
      group,
      idx,
      selected_idx: idx,
      feeds: Vec::new(),
      state: ListState::default().with_selected(Some(0)),
    }
  }
}

impl Component for FeedView {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn handle_key_events(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
    log::info!("{:?} vs {:?}", self.selected_idx, self.idx);
    if self.selected_idx == self.idx {
      let selected_item_idx = self.state.selected().unwrap_or(0);
      match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
          self.state.select(Some((selected_item_idx + 1) % self.feeds.len()));
        },
        KeyCode::Char('k') | KeyCode::Up => {
          if selected_item_idx == 0 {
            self.state.select(Some(self.feeds.len() - 1));
          } else {
            self.state.select(Some(selected_item_idx - 1));
          }
        },
        KeyCode::Char('l') | KeyCode::Enter => {
          if let Some(tx) = &self.command_tx {
            let selected_idx = self.state.selected().unwrap();
            let selected_feed = self.feeds.get(selected_idx).unwrap().clone();
            if selected_feed.id == -1 {
              tx.send(Action::NewTabArticleViewGroup(self.group.clone()))?;
            } else {
              log::info!("Sending NewTabArticleViewFeed");
              tx.send(Action::NewTabArticleViewFeed(selected_feed))?;
            }
          } else {
            log::error!("No tx!")
          }
        },
        _ => {},
      }
    }
    Ok(None)
  }

  fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
    Ok(None)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::ChangeTab(idx) => {
        self.selected_idx = idx;
      },
      Action::RemoveTab(idx) => {
        if self.idx > idx {
          self.idx -= 1;
        }
      },
      Action::UpdateFeedView(idx, feeds) => {
        if self.idx == idx {
          self.feeds = feeds;
        }
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    let name_style = Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::Gray);
    let selected_name_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let selected_desc_style = Style::default().fg(Color::Gray);

    let items: Vec<ListItem> = self
      .feeds
      .iter()
      .enumerate()
      .map(|(i, feed)| {
        if self.state.selected() == Some(i) {
          let text = Text::from(vec![
            Line::styled(&feed.name, selected_name_style),
            Line::styled(&feed.desc, selected_desc_style),
          ]);
          ListItem::new(text)
        } else {
          let text = Text::from(vec![
            Line::styled(&feed.name, name_style),
            Line::styled(&feed.desc, desc_style),
          ]);
          ListItem::new(text)
        }
      })
      .collect();

    let list = List::new(items)
      .block(Block::bordered().border_type(BorderType::Rounded))
      .highlight_symbol(" ┃ ")
      .repeat_highlight_symbol(true);

    f.render_stateful_widget(list, area, &mut self.state);

    Ok(())
  }
}
