use ::plugin::Plugin;
use ::bot::{Connection, Message};

pub struct BullyPlugin;

impl Plugin for BullyPlugin {
    fn new() -> Box<Plugin> {
        Box::new(BullyPlugin{})
    }

    fn is_match(&self, msg: &Message) -> bool {
        // content() clones, so prefer to do it once.
        let content = msg.content();
        content.starts_with("_bully shy imouto") &&
            content.ends_with("_")
    }

    fn handle(&mut self, msg: &Message, conn: &Connection) {
        conn.send(msg, "pls no bully >.<;;;");
    }
}

pub struct HugPlugin;

impl Plugin for HugPlugin {
    fn new() -> Box<Plugin> {
        Box::new(HugPlugin{})
    }

    fn is_match(&self, msg: &Message) -> bool {
        let content = msg.content();
        content.starts_with("_hug shy imouto") &&
            content.ends_with("_")
    }

    fn handle(&mut self, msg: &Message, conn: &Connection) {
        conn.send(msg, &format!("_hug {}_", msg.author().name));
    }
}
