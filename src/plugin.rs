use bot::{Connection, Message};

/// A `Plugin` is a user implemented handler for specific messages. A `Plugin`
/// must implement `Send` as it is shared between threads. It must also return
/// a `Box<Plugin>` to guarantee the `Plugin` owns its own data.
pub trait Plugin: Send {
    fn new() -> Box<Plugin> where Self: Sized;
    fn is_match(&self, message: &Message) -> bool;
    fn handle(&mut self, message: &Message, conn: &Connection);
}

/// A `DefaultPlugin` is an empty struct used for the default implementation of
/// `Plugin::new`. Use the default implementation if you do not need to store
/// any data within a `Plugin`.
pub struct DefaultPlugin;
