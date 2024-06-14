use serde::{
  de::{Deserializer, Visitor},
  Deserialize, Serialize,
};
use strum::Display;

use crate::{
  db::{Feed, FeedItem, Group},
  mode::Mode,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum Action {
  Tick,
  Render,
  Resize(u16, u16),
  Suspend,
  Resume,
  ConfirmQuit,
  Quit,
  RequestRefresh,
  Refresh(Vec<Group>),
  ModeChange(Mode),
  ChangeToFeedList(Group),
  ChangeToFeedView(Group),
  ChangeToFeedViewSingle(Feed),
  RequestUpdateReader(FeedItem),
  UpdateReader(String),
  ActivateReader,
  ActivateFeedList,
  Error(String),
  Help,
}
