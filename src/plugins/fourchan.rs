extern crate clover;
extern crate regex;

use std::error::Error;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ::bot::{Connection, Message};
use ::plugin::Plugin;

pub struct FourchanImagePlugin {
    boards: HashMap<String, clover::Board>,
    client: Arc<Mutex<clover::Client>>,
    img_regex: regex::Regex,
}

impl Plugin for FourchanImagePlugin {
    fn new() -> Box<Plugin> {
        Box::new(FourchanImagePlugin {
            boards: HashMap::new(),
            client: Arc::new(Mutex::new(clover::Client::new().unwrap())),
            img_regex: regex::Regex::new(r"https?://i.4cdn.org/([^/]+)/\d+\.[^\s]+")
                .unwrap()
        })
    }

    fn is_match(&self, msg: &Message) -> bool {
        self.img_regex.is_match(&msg.content())
    }

    // TODO doesn't work. 
    fn handle(&mut self, msg: &Message, conn: &Connection) {
        let content = msg.content();
        let captures = self.img_regex.captures(&content);
        if captures.is_some() {
            let caps = captures.unwrap();
            let url = caps.get(0).map_or("", |u| u.as_str());
            let board_name = caps.get(1).map_or("", |b| b.as_str());

            let empty_b = match clover::Board::new(self.client.to_owned(), board_name) {
                Ok(b) => b,
                Err(_) => return
            };

            let board = self.boards.entry(board_name.to_string()).or_insert(empty_b);
            let _ = board.catalog().unwrap();
            for thread in board.thread_cache.lock().unwrap().threads.values() {
                if thread.image_urls().contains(&url.to_string()) {
                    conn.reply(msg, &format!("posted an image from {}", thread.url()));
                    return
                }
            }
        }
    }
}

pub struct FourchanPlugin {
    boards: HashMap<String, clover::Board>,
    client: Arc<Mutex<clover::Client>>,
    regex: regex::Regex,
}

impl Plugin for FourchanPlugin {
    fn new() -> Box<Plugin> {
        Box::new(FourchanPlugin {
            boards: HashMap::new(),
            client: Arc::new(Mutex::new(clover::Client::new().unwrap())),
            regex: regex::Regex::new(r"!4c\s([A-Za-z0-9]+),(.+)").unwrap()
        })
    }

    fn is_match(&self, msg: &Message) -> bool {
        msg.content().starts_with("!4c ")
    }

    fn handle(&mut self, msg: &Message, conn: &Connection) {
        let content = msg.content();
        let captures = self.regex.captures(&content);
        if captures.is_some() {
            let caps = captures.unwrap();
            let board_name = caps.get(1).map_or("", |b| b.as_str());
            let query = caps.get(2).map_or("", |q| q.as_str());

            let empty_b = match clover::Board::new(self.client.to_owned(),
                                                   board_name) {
                Ok(b) => b,
                Err(e) => {
                    if e.description() == "Invalid board name"{
                        conn.reply(msg, "That's not a board");
                    }
                    return
                }
            };

            let board = self.boards.entry(board_name.to_string())
                .or_insert(empty_b);
            let _ = board.catalog().unwrap();
            let threads = board.find_cached(query.to_string().trim())
                .unwrap().iter()
                .map(|t| format!("{} {}", t.url(), sub_or_com(t)))
                .collect::<Vec<String>>().join("\n");
            if threads.is_empty() {
                conn.reply(msg, &format!("Found no matches for query {} in board {}",
                                         query.to_string().trim(), board_name));
            } else {
                conn.reply(msg, &format!("Found matches for query {}:",
                                         query.to_string().trim()));
                conn.send(msg, &threads.to_string());
            }
        }
    }
}

fn sub_or_com(thread: &clover::Thread) -> String {
    let t = thread.clone();
    if t.topic.sub.is_empty() {
        if t.topic.com.len() > 50 {
            let mut abridged = t.topic.com;
            abridged.truncate(50);
            format!("{}...", abridged)
        } else {
            format!("{}", t.topic.com)
        }
    } else {
        t.topic.sub
    }
}
