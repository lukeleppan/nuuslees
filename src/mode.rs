use serde::{Deserialize, Serialize};

use crate::db::FeedItem;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
  #[default]
  GroupView,
  FeedList,
  ViewArticles(Vec<FeedItem>),
  Refreshing,
}
