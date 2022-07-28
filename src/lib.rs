use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const MAX_PACKET_SIZE: u32 = 10 * 1048576;
const MAX_DURATION_MS: u64 = 5000;

pub struct ServerStatus<'a> {
    pub host: &'a str,
    pub port: u16,
    pub status: Option<Vec<u8>>,
    pub timeout: Duration,
    pub max_size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sample {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    pub name: String,
    pub protocol: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Players {
    pub max: i64,
    pub online: i64,
    #[serde(default)]
    pub sample: Vec<Sample>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TextDescription {
    pub text: String,
}

// servers such as Hypixel send description: String while others send
// description: { text: String }, we must account for both
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Description {
    TextDescription(TextDescription),
    RawDescription(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResponse {
    pub description: Description,
    #[serde(default)]
    pub favicon: String,
    pub players: Players,
    pub version: Version,
}

impl<'a> ServerStatus<'a> {
    pub fn new(host: &'a str, port: u16, timeout: Option<Duration>, max_size: Option<u32>) -> Self {
        Self {
            host,
            port,
            status: None,
            timeout: timeout.unwrap_or(Duration::from_millis(MAX_DURATION_MS)),
            max_size: max_size.unwrap_or(MAX_PACKET_SIZE),
        }
    }

    pub fn query(&mut self) -> Result<(), Box<dyn Error>> {
        let status_packet: Vec<u8> = [
            Self::pack_data(
                &[
                    vec![0x00, 0x00],
                    Self::pack_data(self.host.as_bytes()),
                    self.port.to_be_bytes().into(),
                    vec![0x01],
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<u8>>(),
            ),
            Self::pack_data(&[0x00]),
        ]
        .into_iter()
        .flatten()
        .collect();

        let mut stream = TcpStream::connect_timeout(
            match &format!("{}:{}", self.host, self.port)
                .to_socket_addrs()?
                .next() {
                    Some(v) => v,
                    None => return Err("Invalid address".into()),
                },
            self.timeout,
        )?;
        stream.write_all(&status_packet)?;
        // length of response, not needed
        let _ = Self::read_data(&mut stream)?;
        // id of response, not needed
        let _ = Self::read_data(&mut stream)?;
        let response_length = Self::read_data(&mut stream)?;
        if response_length as u32 > self.max_size {
            return Err("recieved response that was larger than max allowed".into());
        }
        let mut buf = vec![0u8; response_length as usize];
        stream.read_exact(&mut buf)?;
        self.status = Some(buf);
        Ok(())
    }

    pub fn to_json(&self) -> Result<StatusResponse, Box<dyn Error>> {
        let data = match &self.status {
            Some(v) => v,
            None => return Err("no server response, did you forget to query?".into()),
        };
        Ok(serde_json::from_str(&String::from_utf8(data.to_vec())?).expect("parse fail"))
    }

    #[inline(always)]
    fn encode(number: i32) -> Vec<u8> {
        let mut result = Vec::with_capacity((number / 255) as usize);
        let mut val = number;
        while val >= 0x80 {
            result.push(0x80 | (val as u8));
            val >>= 7;
        }
        result.push(val as u8);
        result
    }

    #[inline(always)]
    fn pack_data(data: &[u8]) -> Vec<u8> {
        let mut pack = Self::encode(data.len() as i32);
        pack.extend_from_slice(data);
        pack
    }

    fn read_data(s: &mut TcpStream) -> Result<i32, Box<dyn Error>> {
        let mut len = 0;
        let mut current = vec![0];
        let mut val = 0;

        loop {
            s.read_exact(&mut current)?;
            val |= (current[0] as i32 & 0x7F).checked_shl(len * 7).unwrap_or(0);
            len += 1;
            if len > 5 {
                return Err("server reponse had invaild var int".into());
            }
            if (current[0] & 0x80) != 0x80 {
                break;
            }
        }
        Ok(val)
    }
}
