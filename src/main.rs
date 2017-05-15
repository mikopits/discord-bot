extern crate chrono;
extern crate discord;
extern crate rustc_serialize;

use plugin::Plugin;

mod plugins;
mod bot;
pub mod plugin;

fn main() {
    let mut bot = bot::Bot::new();
    let mut plugins: Vec<Box<Plugin>> = Vec::new();

    plugins.push(plugins::bully::BullyPlugin::new());
    plugins.push(plugins::bully::HugPlugin::new());
    plugins.push(plugins::meme::MemePlugin::new());
    plugins.push(plugins::fourchan::FourchanImagePlugin::new());
    plugins.push(plugins::fourchan::FourchanPlugin::new());
    plugins.push(plugins::anime::AnimePlugin::new());

    for p in plugins {
        bot.register(p);
    }

    bot.connect();
}
