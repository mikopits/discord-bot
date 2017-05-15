extern crate regex;
extern crate nineanime;

use ::plugin::Plugin;
use ::bot::{Connection, Message};

pub struct AnimePlugin {
    regex: regex::Regex,
    last_search: Option<String>,
    last_ep: Option<usize>
}

impl Plugin for AnimePlugin {
    fn new() -> Box<Plugin> {
        Box::new(AnimePlugin{
            regex: regex::Regex::new(r"^!9a\s(\d+),?\s?(.+)").unwrap(),
            last_search: None,
            last_ep: None
        })
    }

    fn is_match(&self, msg: &Message) -> bool {
        self.regex.is_match(&msg.content())
            || msg.content() == "!9a next"
    }

    fn handle(&mut self, msg: &Message, conn: &Connection) {
        let content = msg.content();
        let ep: usize;
        let title: String;

        if content == "!9a next" {
            ep = match self.last_ep {
                Some(ep) => ep + 1,
                None => {
                    conn.reply(msg, "No last episode found");
                    return
                }
            };
            title = match self.last_search {
                Some(ref title) => title.to_string(),
                None => {
                    conn.reply(msg, "No last episode found");
                    return
                }
            }
        } else {
            let caps = match self.regex.captures(&content) {
                Some(c) => c,
                None => {
                    conn.reply(msg, "Could not find any matches");
                    return
                }
            };

            ep = caps.get(1).unwrap().as_str().parse::<usize>().unwrap();
            title = caps.get(2).unwrap().as_str().to_string();
        }

        let matches = nineanime::search(&title).unwrap();
        let anime = match matches.first() {
            Some(a) => a,
            None => {
                conn.reply(msg, "Could not find any matches");
                return
            }
        };
        let files = match anime.files(ep) {
            Ok(f) => f,
            Err(_) => {
                conn.reply(msg, "Could not find any matches");
                return
            }
        };
        let direct_links = files.data.iter()
            .cloned()
            .filter(|d| &*d.label == "720p")
            .map(|d| format!("{} ({})", d.file, d.label))
            .collect::<Vec<String>>();

        conn.reply(msg, "Found links:");
        for link in direct_links {
            conn.send(msg, &link);
        }

        self.last_ep = Some(ep);
        self.last_search = Some(title.clone().to_string());
    }
}
