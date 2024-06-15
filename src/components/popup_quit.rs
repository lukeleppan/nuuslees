use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Modifier, Style};
use ratatui::style::Color;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Wrap};
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
use crate::components::Component;
use crate::config::Config;
use crate::tui::Frame;

pub struct QuitPopup {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  show: bool,
}

impl QuitPopup {
  pub fn new() -> Self {
    Self {
      command_tx: None,
      config: Config::default(),
      show: false,
    }
  }
}

impl Component for QuitPopup {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> color_eyre::Result<()> {
    self.config = config;
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
    if self.show {
      match key.code {
        KeyCode::Char('y') => {
          if let Some(tx) = &self.command_tx {
            tx.send(Action::Quit)?;
          }
        }
        KeyCode::Char('n') => {
          self.show = false;
        }
        _ => {}
      }
    }

    Ok(None)
  }
  fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
    if self.config.confirm_quit {
      match action {
        Action::ConfirmQuit => {
          self.show = true;
        }
        _ => {}
      }
    } else {
      if let Some(tx) = &self.command_tx {
        tx.send(Action::Quit)?;
      }
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> color_eyre::Result<()> {
    if self.show {
      let percent_x: u16 = 40;
      let percent_y: u16 = 20;

      let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
      ]).split(area);
      let popup_area = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2)
      ]).split(popup_layout[1])[1];

      let text = Text::from(vec![
        Line::styled("Are you sure you want to quit?", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Line::styled("[y]es      [n]o", Style::default().fg(Color::Gray)),
      ]).centered();

      let paragraph = Paragraph::new(text.centered()).centered()
        .wrap(Wrap { trim: true })
        .block(Block::bordered().border_type(BorderType::Rounded));
      f.render_widget(Clear, popup_area);
      f.render_widget(paragraph, popup_area);
    }
    Ok(())
  }
}