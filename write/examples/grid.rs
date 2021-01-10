use ::write::send;
use async_std::io;
use std::{convert::TryFrom, error::Error};

#[async_std::main]
async fn main() {
    let stdin = io::stdin();
    let mut buffer = String::new();
    while let Ok(_) = stdin.read_line(&mut buffer).await {
        match (|| -> Result<_, Box<dyn Error>> {
            Ok(<[u8; 2]>::try_from(
                buffer
                    .trim()
                    .split(',')
                    .map(|arg| arg.parse::<u8>())
                    .collect::<Result<Vec<_>, _>>()?
                    .as_slice(),
            )?)
        })() {
            Ok(data) => {
                send(&data[..]).await.unwrap();
            }
            Err(e) => eprintln!("{}", e),
        }
        buffer = String::new();
    }
}
