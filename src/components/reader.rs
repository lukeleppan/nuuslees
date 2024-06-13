use std::default::Default;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use html5ever::{parse_document, ParseOpts};
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

use crate::action::Action;

use super::Component;

#[derive(Default)]
pub struct Reader<'a> {
    content: Option<String>,
    scroll_position: (u16, u16),
    text: Option<Text<'a>>,
    active: bool,
}

impl<'a> Reader<'a> {
    pub fn new() -> Self {
        Self { content: None, scroll_position: (0, 0), text: None, active: false }
    }

    pub fn set_content(&mut self, content: String) {
        self.content = Some(content);
    }

    pub fn build_text(&mut self) {
        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                drop_doctype: true,
                ..Default::default()
            },
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
            }
            NodeData::Text { contents } => {
                let content = contents.borrow();
                spans.push(Span::raw(content.to_string()));
            }
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
                    }
                    "h1" | "h2" | "h3" => {
                        for child in handle.children.borrow().iter() {
                            let mut heading_spans = vec![];
                            self.walk_dom_recursive(child, text, &mut heading_spans);
                            for span in heading_spans.iter_mut() {
                                span.style = Style::default().add_modifier(Modifier::BOLD);
                            }
                            spans.extend(heading_spans);
                        }
                    }
                    "a" => {
                        for child in handle.children.borrow().iter() {
                            let mut link_spans = vec![];
                            self.walk_dom_recursive(child, text, &mut link_spans);
                            for span in link_spans.iter_mut() {
                                span.style = Style::default().fg(Color::Blue);
                            }
                            spans.extend(link_spans);
                        }
                    }
                    _ => {
                        for child in handle.children.borrow().iter() {
                            self.walk_dom_recursive(child, text, spans);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl Component for Reader<'_> {
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Char('k') => {
                if self.scroll_position.1 > 0 {
                    self.scroll_position.1 = self.scroll_position.1 - 1;
                }
            }
            KeyCode::Char('j') => {
                self.scroll_position.1 = self.scroll_position.1 + 1;
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                if self.scroll_position.1 > 0 {
                    self.scroll_position.1 = self.scroll_position.1 - 1;
                }
            }
            MouseEventKind::ScrollDown => {
                self.scroll_position.1 = self.scroll_position.1 + 1;
            }
            _ => {}
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::UpdateReader(content) => {
                self.content = Some(content);
                self.build_text();
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if let Some(text) = self.text.clone() {
            let paragraph = Paragraph::new(text)
                .block(Block::bordered().borders(Borders::ALL))
                .wrap(Wrap { trim: true })
                .scroll(self.scroll_position);
            f.render_widget(paragraph, area);
        }

        Ok(())
    }
}
