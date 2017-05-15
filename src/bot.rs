use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use discord::{Discord, Connection as DiscordConnection, ChannelRef,
             State, Error};
use discord::model::{Message as DiscordMessage, MessageType, Event,
                     ChannelType, MessageId, ChannelId, RoleId, Attachment,
                     MessageReaction, User, ReadyEvent};
use plugin::Plugin;

pub struct Bot {
    conn: Connection,
    // FIXME is this really necessary?
    // Don't think each plugin needs its own Arc.
    // TODO busstop instead of std mutex
    plugins: Arc<Mutex<Vec<Arc<Mutex<Box<Plugin>>>>>>
}

impl Bot {
    /// Creates a new `Bot`.
    pub fn new() -> Self {
        let discord = Discord::from_bot_token(
            &env::var("DISCORD_TOKEN").expect("Expected token"),
        ).expect("Login failed");

        Bot {
            conn: Connection::new(discord),
            plugins: Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn connect(&mut self) {
        let (mut connection, ready) = self.conn.connect()
            .expect("Connect failed");
        let mut state = State::new(ready);
        let channel_count: usize = state.servers().iter()
            .map(|srv| srv.channels.iter()
                 .filter(|chan| chan.kind == ChannelType::Text)
                 .count()
                 ).fold(0, |v, s| v + s);
        println!("[Ready] {} logging {} servers with {} text channels",
                 state.user().username, state.servers().len(), channel_count);

        let plugins = self.plugins.clone();

        loop {
            let event = match connection.recv_event() {
                Ok(event) => event,
                Err(Error::Closed(code, body)) => {
                    println!("[Error] Connection closed with status {:?}: {}",
                             code, body);
                    break
                }
                Err(err) => {
                    println!("[Warning] Receive error: {:?}", err);
                    continue
                }
            };
            state.update(&event);

            match event {
                Event::MessageCreate(message) => {
                    match state.find_channel(message.channel_id) {
                        Some(ChannelRef::Public(server, channel)) => {
                            println!("[{} #{}] {}: {}",
                                     server.name,
                                     channel.name,
                                     message.author.name,
                                     message.content);
                        }
                        Some(ChannelRef::Group(group)) => {
                            println!("[Group {}] {}: {}",
                                     group.name(),
                                     message.author.name,
                                     message.content);
                        }
                        Some(ChannelRef::Private(channel)) => {
                            if message.author.name == channel.recipient.name {
                                println!("[Private] {}: {}",
                                         message.author.name,
                                         message.content);
                            } else {
                                println!("[Private] To {}: {}",
                                         channel.recipient.name,
                                         message.content);
                            }
                        }
                        None => println!("[Unknown Channel] {}: {}",
                                         message.author.name,
                                         message.content),
                    };

                    let msg = Message::new(message);
                    for p in plugins.lock().unwrap().iter()
                        .filter(|&p| p.lock().unwrap().is_match(&msg)) {
                            // FIXME: Don't actually want to clone the plugin
                            // every time it is matched. This is bad because
                            // a plugin could possibly contain a data structure
                            // acting as a database - it will be slow.
                            //
                            // I don't think there is an alternative.
                            let p_1 = p.clone();
                            let conn_1 = self.conn.clone();
                            let msg_1 = msg.clone();
                            println!("[plugin] Spawning thread");
                            thread::spawn(move || {
                                p_1.lock().unwrap().handle(&msg_1, &conn_1);
                            });
                        }
                }
                Event::Unknown(name, data) => {
                    println!("[Unknown Event] {}: {:?}", name, data);
                }
                _ => {}
            }
        }
    }

    pub fn register(&mut self, plugin: Box<Plugin>) {
        self.plugins.lock().unwrap().push(Arc::new(Mutex::new(plugin)));
    }
}

#[derive(Clone)]
pub struct Connection {
    inner: Arc<Mutex<Discord>>
}

impl Connection {
    pub fn new(discord: Discord) -> Connection {
        Connection {
            inner: Arc::new(Mutex::new(discord))
        }
    }

    pub fn connect(&self) -> Result<(DiscordConnection, ReadyEvent), Error> {
        self.inner.lock().unwrap().connect()
    }

    /// Sends a message to the same channel in which the message was received.
    pub fn send(&self, msg: &Message, text: &str) {
        self.inner.lock().unwrap()
            .send_message(msg.channel_id(), text, "", false)
            .expect("Failed to send message");
    }

    /// Sends a message to the same channel in which the message was received.
    /// Prefixes the message with a @mention of the user who sent the message.
    pub fn reply(&self, msg: &Message, text: &str) {
        self.inner.lock().unwrap()
            .send_message(msg.channel_id(),
            &format!("{} {}", msg.author().mention(), text), "", false)
            .expect("Failed to send message");
    }
}

#[derive(Clone)]
pub struct Message {
    inner: Arc<Mutex<DiscordMessage>>
}

impl Message {
    pub fn new(msg: DiscordMessage) -> Message {
        Message {
            inner: Arc::new(Mutex::new(msg))
        }
    }

    pub fn id(&self) -> MessageId {
        self.message().id
    }

    pub fn channel_id(&self) -> ChannelId {
        self.message().channel_id
    }

    pub fn content(&self) -> String {
        self.message().content
    }

    pub fn nonce(&self) -> Option<String> {
        self.message().nonce
    }

    pub fn tts(&self) -> bool {
        self.message().tts
    }

    pub fn timestamp(&self) -> String {
        self.message().timestamp
    }

    pub fn edited_timestamp(&self) -> Option<String> {
        self.message().edited_timestamp
    }

    pub fn pinned(&self) -> bool {
        self.message().pinned
    }

    pub fn kind(&self) -> MessageType {
        self.message().kind
    }

    pub fn author(&self) -> User {
        self.message().author
    }

    pub fn mention_everyone(&self) -> bool {
        self.message().mention_everyone
    }

    pub fn mentions(&self) -> Vec<User> {
        self.message().mentions
    }

    pub fn mention_roles(&self) -> Vec<RoleId> {
        self.message().mention_roles
    }

    pub fn reactions(&self) -> Vec<MessageReaction> {
        self.message().reactions
    }

    pub fn attachments(&self) -> Vec<Attachment> {
        self.message().attachments
    }

    fn message(&self) -> DiscordMessage {
        // FIXME Is there a way to not clone?
        self.inner.lock().unwrap().clone()
    }
}
