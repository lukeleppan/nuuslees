use std::default::Default;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use html5ever::{parse_document, tendril::TendrilSink, tree_builder::TreeBuilderOpts, ParseOpts};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use ratatui::{
  layout::Rect,
  style::{Color, Modifier, Style},
  text::{Line, Span, Text},
  widgets::{Block, Paragraph, Wrap},
  Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::action::Action;

#[derive(Default)]
pub struct ArticleReader<'a> {
  command_tx: Option<UnboundedSender<Action>>,
  idx: usize,
  content: Option<String>,
  scroll_position: (u16, u16),
  text: Option<Text<'a>>,
  active: bool,
}

impl<'a> ArticleReader<'a> {
  pub fn new(idx: usize) -> Self {
    Self {
      command_tx: None,
      idx,
      content: None,
      scroll_position: (0, 0),
      text: None,
      active: false,
    }
  }

  pub fn set_content(&mut self, content: String) {
    self.content = Some(content);
  }

  pub fn build_text(&mut self) {
    let opts = ParseOpts {
      tree_builder: TreeBuilderOpts { drop_doctype: true, ..Default::default() },
      ..Default::default()
    };

    let dom = parse_document(RcDom::default(), opts)
      .from_utf8()
      .read_from(&mut self.content.clone().unwrap().as_bytes())
      .unwrap();

    self.text = Some(self.walk_dom(&dom.document));
  }

  fn walk_dom(&self, handle: &Handle) -> Text<'a> {
    let mut text = Text::default();
    self.walk_dom_recursive(handle, &mut text, &mut vec![]);
    text
  }

  fn walk_dom_recursive(&self, handle: &Handle, text: &mut Text<'a>, spans: &mut Vec<Span<'a>>) {
    match &handle.data {
      NodeData::Document => {
        for child in handle.children.borrow().iter() {
          self.walk_dom_recursive(child, text, spans);
        }
      },
      NodeData::Text { contents } => {
        let content = contents.borrow();
        spans.push(Span::raw(content.to_string()));
      },
      NodeData::Element { name, .. } => {
        let tag_name = name.local.as_ref();

        match tag_name {
          "p" => {
            // Push current spans as a new line if any
            if !spans.is_empty() {
              text.lines.push(Line::from(spans.clone()));
              spans.clear();
            }
            // Process children of <p>
            for child in handle.children.borrow().iter() {
              self.walk_dom_recursive(child, text, spans);
            }
            // Push a new line after the paragraph
            if !spans.is_empty() {
              text.lines.push(Line::from(spans.clone()));
              spans.clear();
            }
            text.lines.push(Line::from(vec![])); // Add an empty line
          },
          "h1" | "h2" | "h3" => {
            for child in handle.children.borrow().iter() {
              let mut heading_spans = vec![];
              self.walk_dom_recursive(child, text, &mut heading_spans);
              for span in heading_spans.iter_mut() {
                span.style = Style::default().add_modifier(Modifier::BOLD);
              }
              spans.extend(heading_spans);
            }
          },
          "a" => {
            for child in handle.children.borrow().iter() {
              let mut link_spans = vec![];
              self.walk_dom_recursive(child, text, &mut link_spans);
              for span in link_spans.iter_mut() {
                span.style = Style::default().fg(Color::Blue);
              }
              spans.extend(link_spans);
            }
          },
          _ => {
            for child in handle.children.borrow().iter() {
              self.walk_dom_recursive(child, text, spans);
            }
          },
        }
      },
      _ => {},
    }
  }
}

impl Component for ArticleReader<'_> {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    if self.active {
      match key.code {
        KeyCode::Char('k') => {
          if self.scroll_position.0 > 0 {
            self.scroll_position.0 = self.scroll_position.0 - 1;
          }
        },
        KeyCode::Char('j') => {
          self.scroll_position.0 = self.scroll_position.0 + 1;
        },
        KeyCode::Char('h') => {
          if let Some(tx) = &self.command_tx {
            tx.send(Action::ActivateFeedList)?;
            self.active = false;
          }
        },
        _ => {},
      }
    }
    Ok(None)
  }

  fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
    if self.active {
      match mouse.kind {
        MouseEventKind::ScrollUp => {
          if self.scroll_position.0 > 0 {
            self.scroll_position.0 = self.scroll_position.0 - 1;
          }
        },
        MouseEventKind::ScrollDown => {
          self.scroll_position.0 = self.scroll_position.0 + 1;
        },
        _ => {},
      }
    }
    Ok(None)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::UpdateReader(idx, content) => {
        if self.idx == idx {
          self.content = Some(content);
          self.build_text();
          self.scroll_position = (0, 0);
        }
      },
      Action::ActivateFeedList => {
        self.active = false;
      },
      Action::ActivateReader => {
        self.active = true;
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    if let Some(text) = self.text.clone() {
      let paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).scroll(self.scroll_position);
      if self.active {
        let paragraph = paragraph
          .block(Block::bordered().style(Style::default().fg(Color::Green)))
          .style(Style::default().fg(Color::White));
        f.render_widget(paragraph, area);
      } else {
        let paragraph = paragraph.block(Block::bordered());
        f.render_widget(paragraph, area);
      }
    }

    Ok(())
  }
}
