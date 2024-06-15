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
  ChangeTab(usize),
  RemoveTab(usize),
  RequestRefresh,
  Refresh(Vec<Group>),
  NewTabFeedView(Group),
  NewTabArticleViewAll,
  NewTabArticleViewGroup(Group),
  NewTabArticleViewFeed(Feed),
  RequestUpdateFeedView(usize, Group),
  RequestUpdateArticleViewAll(usize),
  RequestUpdateArticleViewGroup(usize, Group),
  RequestUpdateArticleViewFeed(usize, Feed),
  UpdateFeedView(usize, Vec<Feed>),
  UpdateArticleView(usize, Vec<FeedItem>),
  ModeChange(Mode),
  RequestUpdateReader(usize, FeedItem),
  UpdateReader(usize, String),
  ActivateReader,
  ActivateFeedList,
  Error(String),
  Help,
}
