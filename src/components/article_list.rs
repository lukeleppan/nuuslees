use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{
  layout::{Margin, Rect},
  prelude::{Color, Line, Modifier, Style, Text},
  widgets::{Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config, db::FeedItem, mode::Mode};

#[derive(Default)]
pub struct ArticleList {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  mode: Mode,
  feed_items: Option<Vec<FeedItem>>,
  selected: usize,
  state: ListState,
  scrollbar_state: ScrollbarState,
  vertical_scroll: usize,
  active: bool,
}

impl ArticleList {
  pub fn new() -> Self {
    Self {
      command_tx: None,
      config: Config::default(),
      mode: Mode::default(),
      feed_items: None,
      selected: 0,
      state: ListState::default().with_selected(Some(0)),
      scrollbar_state: ScrollbarState::default(),
      vertical_scroll: 0,
      active: true,
    }
  }
}

impl Component for ArticleList {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    if self.active {
      if let Some(feed_items) = &self.feed_items {
        let selected_idx = self.state.selected().unwrap_or(0);
        match key.code {
          KeyCode::Char('j') | KeyCode::Down => {
            self.state.select(Some((selected_idx + 1) % feed_items.len()));
          },
          KeyCode::Char('k') | KeyCode::Up => {
            if selected_idx == 0 {
              self.state.select(Some(feed_items.len() - 1));
            } else {
              self.state.select(Some(selected_idx - 1));
            }
          },
          KeyCode::Char('l') | KeyCode::Enter => {
            if let Some(tx) = &self.command_tx {
              let selected_idx = self.state.selected().unwrap();
              let selected_item = feed_items.get(selected_idx).unwrap().clone();
              tx.send(Action::RequestUpdateReader(selected_item))?;
              tx.send(Action::ActivateReader)?;
            }
          },
          _ => {},
        }
      }
    }
    Ok(None)
  }

  fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
    if self.active {
      match self.mode {
        Mode::ViewArticles(_) => {
          if let Some(feed_items) = &self.feed_items {
            let selected_idx = self.state.selected().unwrap_or(0);
            match mouse.kind {
              MouseEventKind::ScrollUp => {
                self.state.select(Some((selected_idx + 1) % feed_items.len()));
              },
              MouseEventKind::ScrollDown => {
                if selected_idx == 0 {
                  self.state.select(Some(feed_items.len() - 1));
                } else {
                  self.state.select(Some(selected_idx - 1));
                }
              },
              _ => {},
            }
          }
        },
        _ => {},
      }
    }
    Ok(None)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      // Action::Refresh(groups) => {
      //   self.groups = Some(groups);
      // },
      Action::ModeChange(mode) => {
        match mode {
          Mode::ViewArticles(feed_items) => {
            self.feed_items = Some(feed_items);
          },
          _ => {},
        }
      },
      Action::ActivateFeedList => {
        self.state.select(Some(0));
        self.active = true;
      },
      Action::ActivateReader => {
        self.state.select(None);
        self.active = false;
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut crate::tui::Frame<'_>, area: Rect) -> Result<()> {
    if let Some(feed_items) = &self.feed_items {
      let name_style = Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD);
      let desc_style = Style::default().fg(Color::Gray);
      let selected_name_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
      let selected_desc_style = Style::default().fg(Color::Gray);

      let items: Vec<ListItem> = feed_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
          if self.state.selected() == Some(i) {
            let text = Text::from(vec![
              Line::styled(&item.title, selected_name_style),
              Line::styled(&item.desc, selected_desc_style),
              // Line::styled("(0/0) read", selected_desc_style),
            ]);
            ListItem::new(text)
          } else {
            let text = Text::from(vec![
              Line::styled(&item.title, name_style),
              Line::styled(&item.desc, desc_style),
              // Line::styled("(0/0) read", desc_style),
            ]);
            ListItem::new(text)
          }
        })
        .collect();

      let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_symbol("┃")
        .repeat_highlight_symbol(true)
        .scroll_padding(1);

      let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(None)
        .thumb_symbol("▌");

      self.scrollbar_state = ScrollbarState::new(list.len()).position(self.state.selected().unwrap_or(0));

      f.render_stateful_widget(list, area, &mut self.state);
      f.render_stateful_widget(
        scrollbar,
        area.inner(&Margin { vertical: 1, horizontal: 0 }),
        &mut self.scrollbar_state,
      );
    } else {
      let block = Block::new().borders(Borders::ALL).title("Feed List");
      f.render_widget(block, area);
    }

    Ok(())
  }
}
