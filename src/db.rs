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
        FOREIGN KEY(feed_id) REFERENCES feeds(id)
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
        log::info!("{:?}", new_group);
        match self.add_group(new_group) {
          Ok(group_id) => {
            for feed in &group.feeds {
              log::info!("{:?}", feed);
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

              log::info!("{:?}", new_feed);

              if let Ok(feed_id) = self.add_feed(new_feed) {
                for item in channel.items() {
                  log::info!("{:?}", item);
                  let content = "".to_string();
                  // if let Some(link) = item.link() {
                  //   content = client.get(link).send().await?.text().await?;
                  // } else {
                  //   content = "".to_string();
                  // }

                  log::info!("{:?}", content);
                  let feed_item = FeedItem {
                    id: 0,
                    feed_id,
                    title: item.title().unwrap_or_default().to_string(),
                    url: item.link().unwrap_or_default().to_string(),
                    desc: item.description().unwrap_or_default().to_string(),
                    content,
                    read: false,
                    pub_date: (item.pub_date().unwrap_or_default())
                      .parse::<chrono::DateTime<Utc>>()
                      .unwrap_or(Utc::now()),
                  };

                  self.add_feed_item(feed_item)?;
                }
              } else {
                log::info!("Failed");
              }
            }
          },
          Err(error) => {
            log::info!("{:?}", error);
          },
        }
      }
    } else {
      log::info!("Failed to get config");
    }
    Ok(())
  }

  pub fn add_group(&self, group: Group) -> Result<i32, DbError> {
    log::info!("Group: {}, {}", group.name, group.desc);
    self.conn.execute("INSERT INTO groups (name, desc) VALUES (?1, ?2)", rusqlite::params![group.name, group.desc])?;
    Ok(self.conn.last_insert_rowid() as i32)
  }

  pub fn get_groups(&self) -> Result<Vec<Group>, DbError> {
    let mut stmt = self.conn.prepare("SELECT * FROM groups")?;
    let group_iter = stmt.query_map([], |row| Ok(Group { id: row.get(0)?, name: row.get(1)?, desc: row.get(2)? }))?;

    let mut groups = Vec::new();
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
      let new_group = Group { id: 0, name: group_name.to_string(), desc: "".to_string() };
      self.add_group(new_group)
    }
  }

  pub fn add_feed(&self, feed: Feed) -> Result<i32, DbError> {
    self.conn.execute(
      "INSERT INTO feeds (group_id, name, desc, url, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
      rusqlite::params![feed.group_id, feed.name, feed.desc, feed.url, feed.updated_at.to_rfc3339()],
    )?;
    Ok(self.conn.last_insert_rowid() as i32)
  }

  pub fn add_feed_item(&self, feed_item: FeedItem) -> Result<i32, DbError> {
    self.conn.execute(
      "INSERT INTO feed_items (feed_id, title, url, desc, read, pub_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
      rusqlite::params![
        feed_item.feed_id,
        feed_item.title,
        feed_item.url,
        feed_item.desc,
        feed_item.read as i32,
        feed_item.pub_date.to_rfc3339()
      ],
    )?;
    Ok(self.conn.last_insert_rowid() as i32)
  }

  pub fn get_feeds(&self) -> Result<Vec<Feed>, DbError> {
    let mut stmt = self.conn.prepare("SELECT id, group_id, name, desc, url, updated_at FROM feeds")?;
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

  pub fn get_feed_items(&self, feed_id: i32) -> Result<Vec<FeedItem>, DbError> {
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
        content: row.get(5)?,
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
}
