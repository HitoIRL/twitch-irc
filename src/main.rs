use irc::client::prelude::*;
use futures::StreamExt;
use serde::{Serialize, Deserialize};
use std::{
    io::{self, Write, Read},
    fs::OpenOptions,
};
use chrono::Local;

const SERVER: &str = "irc.chat.twitch.tv";
const TIME_FORMAT: &str = "%T"; // https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers

#[derive(Serialize, Deserialize)]
struct UserConfig {
    nickname: String,
    oauth: String,
}

#[tokio::main]
async fn main() {
    // open user file (stores nickname and oauth)
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("User.toml"); // todo: move to other direction ("Directions" crate can help)

    let user_config = match file {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            if contents.is_empty() {
                let nickname = get_input("nickname: ");
                let oauth = get_input("oauth: ");

                let user_config = UserConfig {
                    nickname,
                    oauth,
                };

                contents = toml::to_string(&user_config).unwrap();
                file.write_all(contents.as_bytes()).unwrap();
            }

            let data = toml::from_str::<UserConfig>(&contents).unwrap();

            data
        },
        Err(error) => panic!("failed creating user config file: {error}") // should not happen because we use "create" on open options
    };

    let channel = get_input("channel (without #): ");

    let config = Config {
        nickname: Some(user_config.nickname),
        server: Some(SERVER.to_owned()),
        channels: vec![format!("#{channel}")], // todo: allow multiple channels
        password: Some(user_config.oauth),
        port: Some(6697),
        ..Default::default()
    };

    let mut client = Client::from_config(config).await.unwrap();
    client.identify().unwrap();

    let mut stream = client.stream().unwrap();

    // todo: send messages
    while let Ok(Some(message)) = stream.next().await.transpose() {
        if let Command::PRIVMSG(_, ref msg) = message.command { // message on channel
            let user = match message.source_nickname() {
                Some(username) => username,
                None => "?",
            };

            let timestamp = Local::now().format(TIME_FORMAT);

            println!("[{timestamp}] {user}: {msg}"); // todo: colored syntax (eg. nickname, mentions, emotes)
        }
    }
}

fn get_input(message: &str) -> String {
    print!("{message}");
    io::stdout().flush().unwrap();

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    buffer.trim().to_owned()
}
