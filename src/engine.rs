use crate::rpc::{AuthToken, getbalance, pay};
use bitcoincore_zmq::{
    subscribe_single_async,
    Message,
    Message::HashBlock
};
use futures_util::StreamExt;
use std::error::Error;

pub async fn run() {
    let auth = AuthToken::new()
        .target( "127.0.0.1", 51725, "pool_accounting")
        .credentials("user", "password");
    if let Err(e) = listen(&auth).await {
        error!("{}", e);
        std::process::exit(1);
    }
}

async fn scan(auth: &AuthToken) -> Result<(), Box<dyn Error>> {
    let amount = getbalance(auth).await?;
    if amount > 0.02 {
        pay(amount, auth).await?;
    }
    Ok(())
}

async fn listen(auth: &AuthToken) -> Result<(), Box<dyn Error>> {
    let mut stream = subscribe_single_async("tcp://127.0.0.1:28332")?;
    while let Some(msg) = stream.next().await {
        let blockhash = gethash(msg);
        trace!("New block hash: {}", &blockhash);
        scan(auth).await?;
    }
    Ok(())
}

fn gethash<E: Error + Sized>(msg: Result<Message, E>) -> String {
    match msg {
        Ok(msg) => match msg {
            HashBlock(hash, _) => {
                return hash.to_string();
            }
            _ => {
                error!("Got unexpected value from ZMQ.");
                std::process::exit(1);
            }
        },
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    }
}
