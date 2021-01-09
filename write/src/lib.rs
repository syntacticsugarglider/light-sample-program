use std::borrow::Cow;

use surf::{Body, Error};

pub async fn send<'a, T: Into<Cow<'a, [u8]>>>(data: T) -> Result<(), Error> {
    surf::post(&format!(
        "http://lightsmanager.syntacticsugarglider.com/write/{}/192.168.4.203",
        env!("ESP_AUTH_TOKEN")
    ))
    .body(Body::from_bytes(data.into().into_owned()))
    .send()
    .await?;
    Ok(())
}
