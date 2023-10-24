use serde::{Deserialize, Serialize};
use std::env;
use std::{collections::HashMap, process::Command};
use tungstenite::connect;

const WHISPER_ID: &'static str = "WHISPER_ID";
const KAREN_ID: &'static str = "KAREN_ID";
const TAU_TOKEN: &'static str = "TAU_TOKEN";

fn main() {
    let whisper_id = env::var(WHISPER_ID).expect("WHISPER_ID not found in env");
    let karen_id = env::var(KAREN_ID).expect("KAREN_ID not found in env");
    let token = env::var(TAU_TOKEN).expect("TAU_TOKEN not found in env");
    let rewards: HashMap<String, &'static str> =
        HashMap::from([(whisper_id, "whisper"), (karen_id, "karen")]);
    let (mut socket, _response) =
        connect("ws://127.0.0.1:8000/ws/twitch-events/").expect("Can't connect");

    println!("Connected to the server");

    let tua_auth_message = TauAuthMessage {
        token: token.into(),
    };
    socket
        .send(tungstenite::Message::Text(
            serde_json::to_string(&tua_auth_message).expect("failed to serialize tau_auth_message"),
        ))
        .unwrap();
    loop {
        let msg = socket.read().expect("Error reading message");
        if msg.is_text() {
            println!("Received: {}", msg);
            let maybe_message = serde_json::from_str::<Message>(
                &msg.into_text().expect("socket message wasn't text"),
            );
            match maybe_message {
                Ok(msg) => {
                    if rewards
                        .keys()
                        .collect::<Vec<_>>()
                        .contains(&&msg.event_data.reward.id)
                    {
                        let voice = rewards
                            .get(&msg.event_data.reward.id)
                            .copied()
                            .expect("voice should exist");
                        say(voice, msg.event_data.user_input);
                    }
                }
                Err(_) => {}
            }
        }
    }
}

fn say(voice: &str, message: String) {
    let _ = Command::new("/usr/bin/say")
        .args(vec!["-v", voice, &message])
        .output()
        .unwrap();
}

#[derive(Serialize)]
struct TauAuthMessage {
    token: Box<str>,
}

#[derive(Serialize, Deserialize)]
struct Reward {
    id: String,
    cost: u64,
    title: String,
    prompt: String,
}

#[derive(Serialize, Deserialize)]
struct EventData {
    id: String,
    reward: Reward,
    status: String,
    user_id: String,
    user_name: String,
    user_input: String,
    redeemed_at: String,
    broadcaster_user_id: String,
    broadcaster_user_name: String,
    broadcaster_user_login: String,
}

#[derive(Serialize, Deserialize)]
struct Message {
    id: Option<String>,
    event_id: String,
    event_type: String,
    event_source: String,
    event_data: EventData,
    created: String,
    origin: String,
}
