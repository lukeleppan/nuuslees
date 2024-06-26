use std::path::Path;

use chrono::Utc;
use reqwest::Client;
use rusqlite::{Connection, ErrorCode, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;

#[derive(Error, Debug)]
pub enum DbError {
  #[error("Database error")]
  RusqliteError(#[from] rusqlite::Error),

  #[error("Network error")]
  ReqwestError(#[from] reqwest::Error),

  #[error("RSS error")]
  RssError(#[from] rss::Error),

  #[error("Custom error: {0}")]
  Custom(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Group {
  pub id: i32,
  pub name: String,
  pub desc: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Feed {
  pub id: i32,
  pub group_id: i32,
  pub name: String,
  pub desc: String,
  pub url: String,
  pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeedItem {
  pub id: i32,
  pub feed_id: i32,
  pub title: String,
  pub url: String,
  pub desc: String,
  pub content: String,
  pub read: bool,
  pub pub_date: chrono::DateTime<Utc>,
}

pub struct Database {
  conn: Connection,
  config: Option<Config>,
}

impl Database {
  pub async fn new(data_dir: &str) -> Result<Self> {
    let db_path = format!("{data_dir}/nuuslees.db");
    let conn = Connection::open(db_path)?;
    Ok(Self { conn, config: None })
  }

  pub fn set_config(&mut self, config: Config) {
    self.config = Some(config);
  }

  pub async fn init(&self) -> Result<()> {
    self.conn.execute(
      "CREATE TABLE IF NOT EXISTS groups (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        desc TEXT,
        UNIQUE(name)
      )",
      [],
    )?;
    self.conn.execute(
      "CREATE TABLE IF NOT EXISTS feeds (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        group_id INTEGER NOT NULL,
        name TEXT NOT NULL,
        desc TEXT,
        url TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        FOREIGN KEY(group_id) REFERENCES groups(id),
        UNIQUE(url)
      )",
      [],
    )?;
    self.conn.execute(
      "CREATE TABLE IF NOT EXISTS feed_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        feed_id INTEGER NOT NULL,
        title TEXT NOT NULL,
        url TEXT,
        desc TEXT,
        content TEXT,
        read INTEGER NOT NULL,
        pub_date TEXT NOT NULL,
        FOREIGN KEY(feed_id) REFERENCES feeds(id),
        UNIQUE(url)
      )",
      [],
    )?;

    Ok(())
  }

  pub async fn refresh_feeds(&self) -> Result<(), DbError> {
    let client = Client::new();

    if let Some(config) = &self.config {
      for group in &config.groups {
        let new_group = Group { id: 0, name: group.name.clone(), desc: group.desc.clone() };
        let group_id = match self.upsert_group(new_group) {
          Ok(id) => id,
          Err(error) => {
            log::error!("Failed to upsert group: {:?}", error);
            continue;
          },
        };

        for feed in &group.feeds {
          let content = client.get(&feed.link).send().await?.text().await?;
          let channel = rss::Channel::read_from(content.as_bytes())?;

          let new_feed = Feed {
            id: 0, // Placeholder
            group_id,
            name: feed.name.clone().unwrap_or(channel.title().to_string()),
            desc: feed.desc.clone().unwrap_or(channel.description().to_string()),
            url: feed.link.clone(),
            updated_at: Utc::now(),
          };

          let feed_id = match self.upsert_feed(new_feed) {
            Ok(id) => id,
            Err(error) => {
              log::error!("Failed to upsert feed: {:?}", error);
              continue;
            },
          };

          for item in channel.items() {
            let content = "".to_string();

            let feed_item = FeedItem {
              id: 0,
              feed_id,
              title: item.title().unwrap_or_default().to_string(),
              url: item.link().unwrap_or_default().to_string(),
              desc: item.description().unwrap_or_default().to_string(),
              content,
              read: false,
              pub_date: item
                .pub_date()
                .unwrap_or_default()
                .parse::<chrono::DateTime<Utc>>()
                .unwrap_or(Utc::now()),
            };

            match self.upsert_feed_item(feed_item) {
              Ok(_) => (),
              Err(error) => log::error!("Failed to upsert feed item: {:?}", error),
            }
          }
        }
      }
    } else {
      log::error!("Failed to get config");
    }
    Ok(())
  }

  pub fn upsert_group(&self, group: Group) -> Result<i32, DbError> {
    self.conn.execute(
      "INSERT INTO groups (name, desc) VALUES (?1, ?2)
            ON CONFLICT(name) DO UPDATE SET desc=excluded.desc",
      rusqlite::params![group.name, group.desc],
    )?;
    Ok(self.conn.last_insert_rowid() as i32)
  }

  pub fn upsert_feed(&self, feed: Feed) -> Result<i32, DbError> {
    self.conn.execute(
      "INSERT INTO feeds (group_id, name, desc, url, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(url) DO UPDATE SET name=excluded.name, desc=excluded.desc, updated_at=excluded.updated_at",
      rusqlite::params![feed.group_id, feed.name, feed.desc, feed.url, feed.updated_at.to_rfc3339()],
    )?;
    Ok(self.conn.last_insert_rowid() as i32)
  }

  pub fn upsert_feed_item(&self, feed_item: FeedItem) -> Result<i32, DbError> {
    self.conn.execute(
      "INSERT INTO feed_items (feed_id, title, url, desc, content, read, pub_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(url) DO UPDATE SET title=excluded.title, desc=excluded.desc, content=excluded.content, read=excluded.read, pub_date=excluded.pub_date",
      rusqlite::params![
                feed_item.feed_id,
                feed_item.title,
                feed_item.url,
                feed_item.desc,
                feed_item.content,
                feed_item.read as i32,
                feed_item.pub_date.to_rfc3339()
            ],
    )?;
    Ok(self.conn.last_insert_rowid() as i32)
  }

  pub fn get_groups(&self) -> Result<Vec<Group>, DbError> {
    let mut stmt = self.conn.prepare("SELECT * FROM groups")?;
    let group_iter = stmt
      .query_map([], |row| Ok(Group { id: row.get(0)?, name: row.get(1)?, desc: row.get(2)? }))?;

    let all_group = Group {
      id: -1,
      name: "All Feeds".to_string(),
      desc: "See all feeds in all groups".to_string(),
    };
    let mut groups = vec![all_group];
    for group in group_iter {
      groups.push(group?);
    }
    Ok(groups)
  }

  pub fn get_group_id(&self, group_name: &str) -> Result<i32, DbError> {
    let mut stmt = self.conn.prepare("SELECT id FROM groups WHERE name = ?1")?;
    let mut rows = stmt.query([group_name])?;
    if let Some(row) = rows.next()? {
      Ok(row.get(0)?)
    } else {
      Ok(-1)
    }
  }

  pub fn get_feeds(&self) -> Result<Vec<Feed>, DbError> {
    let mut stmt =
      self.conn.prepare("SELECT id, group_id, name, desc, url, updated_at FROM feeds")?;
    let feed_iter = stmt.query_map([], |row| {
      Ok(Feed {
        id: row.get(0)?,
        group_id: row.get(1)?,
        name: row.get(2)?,
        desc: row.get(3)?,
        url: row.get(4)?,
        updated_at: row.get::<_, String>(5)?.parse::<chrono::DateTime<Utc>>().unwrap(),
      })
    })?;

    let mut feeds = Vec::new();
    for feed in feed_iter {
      feeds.push(feed?);
    }
    Ok(feeds)
  }

  pub fn get_feed_items(&self) -> Result<Vec<FeedItem>, DbError> {
    let mut stmt = self
      .conn
      .prepare("SELECT id, feed_id, title, url, desc, content, read, pub_date FROM feed_items")?;

    let feed_item_iter = stmt.query_map([], |row| {
      Ok(FeedItem {
        id: row.get(0)?,
        feed_id: row.get(1)?,
        title: row.get(2)?,
        url: row.get(3)?,
        desc: row.get(4)?,
        content: "".to_string(),
        read: row.get::<_, i32>(6)? != 0,
        pub_date: row.get::<_, String>(7)?.parse::<chrono::DateTime<Utc>>().unwrap(),
      })
    })?;

    let mut feed_items = Vec::new();
    for feed_item in feed_item_iter {
      feed_items.push(feed_item?);
    }
    Ok(feed_items)
  }

  pub fn get_feed_items_from_feed(&self, feed_id: i32) -> Result<Vec<FeedItem>, DbError> {
    let mut stmt = self
      .conn
      .prepare("SELECT id, feed_id, title, url, desc, content, read, pub_date FROM feed_items WHERE feed_id = ?1")?;

    let feed_item_iter = stmt.query_map([feed_id], |row| {
      Ok(FeedItem {
        id: row.get(0)?,
        feed_id: row.get(1)?,
        title: row.get(2)?,
        url: row.get(3)?,
        desc: row.get(4)?,
        content: "".to_string(),
        read: row.get::<_, i32>(6)? != 0,
        pub_date: row.get::<_, String>(7)?.parse::<chrono::DateTime<Utc>>().unwrap(),
      })
    })?;

    let mut feed_items = Vec::new();
    for feed_item in feed_item_iter {
      feed_items.push(feed_item?);
    }
    Ok(feed_items)
  }

  pub fn get_feed_items_from_group(&self, group_id: i32) -> Result<Vec<FeedItem>, DbError> {
    let mut stmt = self.conn.prepare(
      "SELECT feed_items.*
           FROM feed_items
           JOIN feeds ON feed_items.feed_id = feeds.id
           WHERE feeds.group_id = ?",
    )?;

    let feed_item_iter = stmt.query_map([group_id], |row| {
      Ok(FeedItem {
        id: row.get(0)?,
        feed_id: row.get(1)?,
        title: row.get(2)?,
        url: row.get(3)?,
        desc: row.get(4)?,
        content: "".to_string(),
        read: row.get::<_, i32>(6)? != 0,
        pub_date: row.get::<_, String>(7)?.parse::<chrono::DateTime<Utc>>().unwrap(),
      })
    })?;

    let mut feed_items = Vec::new();
    for feed_item in feed_item_iter {
      feed_items.push(feed_item?);
    }
    Ok(feed_items)
  }

  pub fn get_feeds_from_group(&self, group_id: i32) -> Result<Vec<Feed>, DbError> {
    let mut stmt = self
      .conn
      .prepare("SELECT id, group_id, name, desc, url, updated_at FROM feeds WHERE group_id = ?1")?;
    let feed_iter = stmt.query_map(rusqlite::params![group_id], |row| {
      Ok(Feed {
        id: row.get(0)?,
        group_id: row.get(1)?,
        name: row.get(2)?,
        desc: row.get(3)?,
        url: row.get(4)?,
        updated_at: row.get::<_, String>(5)?.parse::<chrono::DateTime<Utc>>().unwrap(),
      })
    })?;

    let all_feed = Feed {
      id: -1,
      group_id,
      name: "All Feeds".to_string(),
      desc: "See all feeds in this group".to_string(),
      url: String::new(),
      updated_at: chrono::Utc::now(),
    };
    let mut feeds = vec![all_feed];
    for feed in feed_iter {
      feeds.push(feed?);
    }
    Ok(feeds)
  }
}
