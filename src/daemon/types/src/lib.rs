use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    convert::TryInto,
    io::{Read, Write},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const BUF_SIZE: usize = 128;

pub async fn async_serialize_into(
    mut w: impl AsyncWrite + Unpin,
    v: impl Serialize,
) -> bincode::Result<()> {
    let size = bincode::serialized_size(&v)? as u32;
    w.write_all(&size.to_le_bytes()).await?;
    let mut buf = [0; BUF_SIZE];
    bincode::serialize_into(&mut buf[..], &v)?;
    w.write_all(&buf[..size as usize]).await?;
    w.flush().await?;
    Ok(())
}

pub async fn async_deserialize_from<R: DeserializeOwned>(
    mut r: impl AsyncRead + Unpin,
) -> bincode::Result<R> {
    let mut buf = [0; BUF_SIZE];
    r.read_exact(&mut buf[..4]).await?;
    let size = u32::from_ne_bytes(buf[0..4].try_into().unwrap()) as usize;
    r.read_exact(&mut buf[..size]).await?;
    bincode::deserialize(&buf)
}

pub fn serialize_into(mut w: impl Write, v: impl Serialize) -> bincode::Result<()> {
    let size = bincode::serialized_size(&v)? as u32;
    w.write_all(&size.to_le_bytes())?;
    let mut buf = [0; BUF_SIZE];
    bincode::serialize_into(&mut buf[..], &v)?;
    w.write_all(&buf[..size as usize])?;
    w.flush()?;
    Ok(())
}

pub fn deserialize_from<R: DeserializeOwned>(mut r: impl Read) -> bincode::Result<R> {
    let mut buf = [0; BUF_SIZE];
    r.read_exact(&mut buf[..4])?;
    let size = u32::from_ne_bytes(buf[0..4].try_into().unwrap()) as usize;
    r.read_exact(&mut buf[..size])?;
    bincode::deserialize(&buf)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IndicatorMessage {
    UpdateHangulState(bool),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum WindowMessage {
    SpawnPreeditWindow { x: u32, y: u32, ch: char },
    RemovePreeditWindow,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientRequest {
    GetGlobalHangulState,
    Indicator(IndicatorMessage),
    Window(WindowMessage),
}

#[derive(Serialize, Deserialize)]
pub struct GetGlobalHangulStateReply(pub bool);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
