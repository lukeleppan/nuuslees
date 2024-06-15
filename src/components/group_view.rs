use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{
  layout::Rect,
  style::{Color, Modifier, Style},
  text::{Line, Text},
  widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config, db::Group, mode::Mode, tui::Frame};

pub struct GroupView {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  mode: Mode,
  idx: usize,
  selected_idx: usize,
  groups: Vec<Group>,
  state: ListState,
}

impl GroupView {
  pub fn new() -> Self {
    Self {
      command_tx: None,
      config: Config::default(),
      mode: Mode::default(),
      idx: 0,
      selected_idx: 0,
      groups: Vec::new(),
      state: ListState::default().with_selected(Some(0)),
    }
  }
}

impl Component for GroupView {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
    self.config = config;
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
    if self.selected_idx == self.idx {
      let selected_item_idx = self.state.selected().unwrap_or(0);
      match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
          self.state.select(Some((selected_item_idx + 1) % self.groups.len()));
        },
        KeyCode::Char('k') | KeyCode::Up => {
          if selected_item_idx == 0 {
            self.state.select(Some(self.groups.len() - 1));
          } else {
            self.state.select(Some(selected_item_idx - 1));
          }
        },
        KeyCode::Char('l') | KeyCode::Enter => {
          if let Some(tx) = &self.command_tx {
            let selected_idx = self.state.selected().unwrap();
            let selected_group = self.groups.get(selected_idx).unwrap().clone();
            if selected_group.id == -1 {
              tx.send(Action::NewTabArticleViewAll)?;
            } else {
              tx.send(Action::NewTabFeedView(selected_group))?;
            }
          }
        },
        _ => {},
      }
    }
    Ok(None)
  }

  fn handle_mouse_events(&mut self, mouse: MouseEvent) -> color_eyre::Result<Option<Action>> {
    let selected_idx = self.state.selected().unwrap_or(0);
    match mouse.kind {
      MouseEventKind::ScrollUp => {
        self.state.select(Some((selected_idx + 1) % self.groups.len()));
      },
      MouseEventKind::ScrollDown => {
        if selected_idx == 0 {
          self.state.select(Some(self.groups.len() - 1));
        } else {
          self.state.select(Some(selected_idx - 1));
        }
      },
      _ => {},
    }
    Ok(None)
  }

  fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
    match action {
      Action::Refresh(groups) => {
        self.groups = groups;
      },
      Action::ChangeTab(idx) => {
        self.selected_idx = idx;
      },
      Action::ModeChange(mode) => {
        self.mode = mode;
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> color_eyre::Result<()> {
    let name_style = Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::Gray);
    let selected_name_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let selected_desc_style = Style::default().fg(Color::Gray);

    let items: Vec<ListItem> = self
      .groups
      .iter()
      .enumerate()
      .map(|(i, group)| {
        if self.state.selected() == Some(i) {
          let text = Text::from(vec![
            Line::styled(&group.name, selected_name_style),
            Line::styled(&group.desc, selected_desc_style),
            Line::styled("(0/0) read", selected_desc_style),
          ]);
          ListItem::new(text)
        } else {
          let text = Text::from(vec![
            Line::styled(&group.name, name_style),
            Line::styled(&group.desc, desc_style),
            Line::styled("(0/0) read", desc_style),
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
