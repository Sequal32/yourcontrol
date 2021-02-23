mod events;

use base64::{decode, encode};
use discord_game_sdk::{Activity, Discord};
use events::YourControlsDiscordEvents;
use std::{net::SocketAddr, str::FromStr, time::SystemTime};

use yourcontrols_types::Error;

type Secret = String;

pub enum JoinMethod {
    SessionCode(String),
    Ip(SocketAddr),
}

pub enum DiscordEvent {
    Join { method: JoinMethod },
    Invited { secret: Secret },
}

// Write unit tests for this struct
struct SecretEncoder {}

impl SecretEncoder {
    fn encode_ip(addr: SocketAddr) -> Secret {
        encode(addr.to_string().as_bytes())
    }

    fn encode_session_id(session_id: String) -> Secret {
        encode(session_id.as_bytes())
    }

    fn decode_secret(secret: &str) -> Result<JoinMethod, Error> {
        let decode_bytes = decode(secret)?;
        let decode_s = String::from_utf8(decode_bytes)?;

        Ok(match SocketAddr::from_str(&decode_s) {
            Ok(addr) => JoinMethod::Ip(addr),
            Err(_) => JoinMethod::SessionCode(decode_s),
        })
    }
}

pub struct YourControlsDiscord<'a> {
    discord: Discord<'a, YourControlsDiscordEvents>,
    activity: Activity,
}

impl<'a> YourControlsDiscord<'a> {
    pub fn new(client_id: i64) -> Self {
        let mut discord = Discord::<YourControlsDiscordEvents>::new(client_id).unwrap();

        *discord.event_handler_mut() = Some(YourControlsDiscordEvents::new());

        Self {
            discord,
            activity: Activity::empty(),
        }
    }
    pub fn do_callbacks(&mut self) {
        self.discord.run_callbacks();
    }
    pub fn get_pending_events(&mut self) -> Option<DiscordEvent> {
        match self
            .discord
            .event_handler()
            .as_ref()
            .unwrap()
            .get_receiver()
            .try_recv()
        {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }
    pub fn update_activity(&mut self) {
        self.discord.update_activity(&self.activity, |_, _| {})
    }

    pub fn get_activity_mut(&mut self) -> &mut Activity {
        &mut self.activity
    }

    pub fn set_current_time(&mut self) {
        self.activity.with_start_time(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bad_secret() {
        assert!(SecretEncoder::decode_secret("oof").is_err());
    }

    #[test]
    fn test_encode_decode() {
        let encoded = SecretEncoder::encode_session_id("safesucks".to_string());
        let output = JoinMethod::SessionCode("safesucks".to_string());
        assert!(std::matches!(
            SecretEncoder::decode_secret(&encoded),
            Ok(output)
        ));
    }

    #[test]
    fn test_encode_decode_ip() {
        let addr: SocketAddr = "127.0.0.1:23213".parse().unwrap();
        let encoded = SecretEncoder::encode_ip(addr);
        let output = JoinMethod::Ip(addr);
        assert!(std::matches!(
            SecretEncoder::decode_secret(&encoded),
            Ok(output)
        ));
    }
}