use futures::prelude::*;
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

    for tag in message.tags.as_ref().unwrap().iter() {
        let Tag(key, value) = tag;
        if key == "display-name" && value.is_some() {
            nickname = value.as_ref().unwrap().to_owned();
        } else if key == "id" && value.is_some() {
            id = value.as_ref().unwrap().to_owned();
        }
    }

    TwitchCommand::Message {
        id: id,
        username: nickname,
        message: text.to_owned(),
    }
}
