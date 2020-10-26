use futures::prelude::*;
use hex::FromHex;
use irc::client::prelude::*;
use irc::proto::message::Tag;
use rand::prelude::*;
use std::sync::mpsc::Sender;

pub enum TwitchCommand {
    ClearChat {
        user: Option<String>,
    },
    ClearMsg {
        id: String,
    },
    Message {
        id: String,
        username: String,
        message: String,
        color: (u8, u8, u8),
    },
}

pub async fn connect_to_chat(channel: &str, tx: Sender<TwitchCommand>) -> irc::error::Result<()> {
    let nick = String::from("justinfan") + &random::<u32>().to_string();
    let config = Config {
        nickname: Some(nick),
        server: Some("irc.chat.twitch.tv".to_owned()),
        channels: vec![channel.to_owned()],
        ..Config::default()
    };

    let mut client = Client::from_config(config).await?;
    let capabilities = [
        Capability::Custom("twitch.tv/tags"),
        Capability::Custom("twitch.tv/commands"),
    ];
    client.send_cap_req(&capabilities)?;
    client.identify()?;

    let mut stream = client.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        match &message.command {
            Command::PRIVMSG(_, msg) => {
                tx.send(parse_privmsg(&message, msg)).unwrap();
            }
            _ => {}
        }
    }

    Ok(())
}

fn parse_privmsg(message: &Message, text: &str) -> TwitchCommand {
    let mut nickname = match message.source_nickname() {
        Some(name) => name.to_owned(),
        None => String::new(),
    };
    let mut id = String::new();
    let mut color: Vec<u8> = vec![255, 255, 255];

    for tag in message.tags.as_ref().unwrap().iter() {
        let Tag(key, value) = tag;
        if value.is_some() {
            match key.as_str() {
                "display-name" => nickname = value.as_ref().unwrap().to_owned(),
                "id" => id = value.as_ref().unwrap().to_owned(),
                "color" => {
                    let hex = value.as_ref().unwrap();
                    // color can be an empty string
                    if hex.len() > 0 {
                        let hex = hex[1..].to_owned();
                        color = Vec::from_hex(&hex).unwrap();
                    }
                }
                _ => {}
            }
        }
    }

    TwitchCommand::Message {
        id: id,
        username: nickname,
        message: text.to_owned(),
        color: (color[0], color[1], color[2]),
    }
}
