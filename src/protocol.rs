use serde::{Serialize, Deserialize};
use bon::Builder;
use smol::net::TcpStream;
use smol::io::AsyncWriteExt;

pub trait Message:
    Default + Send + Sync + Clone + Serialize + for<'de> Deserialize<'de> + 'static + std::fmt::Debug + PartialEq
{
}

pub const V0: u8 = 0;
pub const FORM_REQUEST: u8 = 0;
pub const FORM_RESPONSE: u8 = 1;
pub const HEADER_LENGTH: u8 = 6;


#[derive(Builder, Debug, Serialize, Deserialize)]
pub struct Header {
    version: u8,
    form: u8,
    pub length: u16,
    reserved: u16,
}

impl Header {
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(self)?)
    }

    pub fn from_bytes(buffer: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize::<Self>(buffer)?)
    }
}

pub async fn send_packet(stream: &mut TcpStream, header: Header, payload: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let header_bytes = header.to_bytes()?;
    stream.write_all(&header_bytes).await?;
    stream.write_all(&payload).await?;
    Ok(())
}
