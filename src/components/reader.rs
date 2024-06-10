use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  widgets::{Block, Borders, Paragraph, Tabs, Wrap},
  Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::action::Action;

#[derive(Default)]
pub struct Reader {
  content: Option<String>,
  scroll_position: (u16, u16),
}

impl Reader {
  pub fn new() -> Self {
    Self { content: None, scroll_position: (0, 0) }
  }

  pub fn set_content(&mut self, content: String) {
    self.content = Some(content);
  }
}

impl Component for Reader {
  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    match key.code {
      KeyCode::Char('k') => {
        if self.scroll_position.1 > 0 {
          self.scroll_position.1 = self.scroll_position.1 - 1;
        }
      },
      KeyCode::Char('j') => {
        self.scroll_position.1 = self.scroll_position.1 + 1;
      },
      _ => {},
    }
    Ok(None)
  }

  fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
    match mouse.kind {
      MouseEventKind::ScrollUp => {
        if self.scroll_position.1 > 0 {
          self.scroll_position.1 = self.scroll_position.1 - 1;
        }
      },
      MouseEventKind::ScrollDown => {
        self.scroll_position.1 = self.scroll_position.1 + 1;
      },
      _ => {},
    }
    Ok(None)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::UpdateReader(content) => {
        self.content = Some(content);
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    if let Some(content) = self.content.clone() {
      let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll(self.scroll_position);
      f.render_widget(paragraph, area);
    }

    Ok(())
  }
}
