extern crate chrono;
extern crate csv;
extern crate discord;
extern crate rand;
extern crate rustc_serialize;

use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;

use chrono::{DateTime, Duration, UTC};
use discord::model::UserId;
use plugins::rand::Rng;
use ::bot::{Connection, Message};
use ::plugin::Plugin;

static FILE_PATH: &'static str = "memelist.csv";

#[derive(Clone, RustcEncodable, RustcDecodable)]
struct Meme {
    date: DateTime<UTC>,
    author: String,
    content: String,
}

pub struct MemePlugin {
    file: File,
    memes: Vec<Meme>,
    cooldown: Duration,
    //ban_duration: Duration,
    last_used_map: BTreeMap<UserId, DateTime<UTC>>,
    last_meme: Option<Meme>,
}

impl Plugin for MemePlugin {
    fn new() -> Box<Plugin> {
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(FILE_PATH)
            .expect("Failed to open file");

        let mut rdr = csv::Reader::from_file(FILE_PATH)
            .expect("Failed to read file")
            .has_headers(false);
        let memes = rdr.decode()
            .collect::<csv::Result<Vec<Meme>>>()
            .expect("Failed to decode");

        Box::new(MemePlugin {
            file: file,
            memes: memes,
            cooldown: Duration::seconds(60),
            //ban_duration: Duration::minutes(15),
            last_used_map: BTreeMap::new(),
            last_meme: None,
        })
    }

    fn is_match(&self, msg: &Message) -> bool {
        let content = msg.content();
        content.starts_with("!meme") ||
            content == "!info"
    }

    fn handle(&mut self, msg: &Message, conn: &Connection) {
        let content = msg.content();
        // Get a meme
        if content == "!meme" {
            if self.is_banned(msg, conn) { return };
            let meme = rand::thread_rng().choose(&self.memes).unwrap();
            self.last_meme = Some(meme.clone());
            conn.reply(msg, &meme.content);
        }

        // Add a meme
        else if content.starts_with("!meme ") {
            if self.is_banned(msg, conn) { return };
            let meme = Meme {
                date: UTC::now(),
                author: msg.author().name,
                content: String::from(&content[6..]),
            };
            let mut wtr = csv::Writer::from_memory();
            wtr.encode(meme.clone()).expect("Failed to encode");
            self.file.write_all(wtr.as_bytes())
                .expect("Failed to write to file");
            self.memes.push(meme.clone());
            conn.reply(msg, &format!("{} is now a meme", meme.content));
        }

        // Get meme info
        else if content == "!memeinfo" || content == "!info" {
            match self.last_meme.clone() {
                None => return,
                Some(m) => {
                    conn.reply(msg, &format!(
                        "This meme was added by {} at {}",
                        m.author, m.date.to_rfc2822()));
                }
            }
        }
    }
}

impl MemePlugin {
    fn is_banned(&mut self, msg: &Message, conn: &Connection) -> bool {
        let now = UTC::now();
        let author = msg.author();
        if self.last_used_map.contains_key(&author.id) {
            if now.signed_duration_since(
                *self.last_used_map
                .get(&author.id)
                .unwrap()) <= self.cooldown {
                conn.reply(msg, "whoa there, slow down desu senpai");
                return true
            }
        }
        self.last_used_map.insert(author.id, now);
        false
    }
}
