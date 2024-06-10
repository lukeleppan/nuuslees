use std::{fmt, string::ToString};

use serde::{
  de::{self, Deserializer, Visitor},
  Deserialize, Serialize,
};
use strum::Display;

use crate::{
  db::{FeedItem, Group},
  mode::Mode,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum Action {
  Tick,
  Render,
  Resize(u16, u16),
  Suspend,
  Resume,
  Quit,
  ChangeToFeedView(Group),
  ModeChange(Mode),
  RequestRefresh,
  Refresh(Vec<Group>),
  RequestUpdateReader(FeedItem),
  UpdateReader(String),
  Error(String),
  Help,
}
