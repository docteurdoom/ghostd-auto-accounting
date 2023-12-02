use crate::math;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

#[derive(Debug, Clone, Default)]
pub struct AuthToken {
    user: String,
    password: String,
    url: String,
}

impl AuthToken {
    pub fn new() -> Self {
        Self {
            user: String::new(),
            password: String::new(),
            url: String::new(),
        }
    }
    pub fn target(mut self, ip: &str, port: u16, walletname: &str) -> Self {
        debug!("Generating auth ...");
        if walletname.len() == 0 {
            self.url = format!("http://{}:{}/", ip, port);
        } else {
            self.url = format!("http://{}:{}/wallet/{}", ip, port, walletname);
        }
        return self;
    }
    pub fn credentials(mut self, user: impl Into<String>, password: impl Into<String>) -> Self {
        trace!("Registering credentials ...");
        self.user = user.into();
        self.password = password.into();
        return self;
    }
}

fn parametrize(args: &str) -> Vec<Value> {
    trace!("Parsing arguments ...");
    let mut params: Vec<Value> = Vec::new();
    for entry in args.split(" ").collect::<Vec<&str>>() {
        match serde_json::from_str(entry) {
            Ok(val) => {
                params.push(val);
            }
            Err(_) => {
                params.push(Value::String(entry.to_string()));
            }
        }
    }
    return params;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RPCResponse {
    pub result: Value,
    pub error: Option<String>,
    pub id: String,
}

impl RPCResponse {
    fn unpack(self) -> Value {
        match self.error {
            Some(err) => {
                error!("{}", err);
                std::process::exit(1);
            }
            None => self.result,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Post<'r> {
    jsonrpc: &'r str,
    id: &'r str,
    method: Value,
    params: Value,
}

pub(crate) async fn call(args: &str, auth: &AuthToken) -> Result<Value, Box<dyn Error>> {
    let mut params = parametrize(args);
    let method = params[0].clone();
    params.remove(0);

    let post = Post {
        jsonrpc: "",
        id: "",
        method,
        params: Value::Array(params),
    };
    debug!("RPC: {} {} ...", &post.method, &post.params);
    let response = reqwest::Client::new()
        .post(auth.url.clone())
        .basic_auth(auth.user.clone(), Some(auth.password.clone()))
        .json(&post)
        .send()
        .await;
    match response {
        Ok(context) => {
            let rpcresponse: RPCResponse = context.json().await?;
            let json = rpcresponse.unpack();
            return Ok(json);
        }
        Err(err) => {
            error!("{}", err);
            std::process::exit(1);
        }
    }
}

pub async fn getbalance(auth: &AuthToken) -> Result<f64, Box<dyn Error>> {
    debug!("Retreiving balance ...");
    let raw = call("getbalance \"*\" 1", auth).await?;
    let amount = serde_json::from_value::<f64>(raw)?;
    Ok(math::precise(amount))
}

pub async fn pay(amount: f64, auth: &AuthToken) -> Result<String, Box<dyn Error>> {
    let amount = amount - 0.01;
    let mut recipients: HashMap<&str, f64> = HashMap::new();
    recipients.insert("GaMd2KZVkuxGMLoJDzQMsbPVajoCFd4RS9", math::precise(0.6*amount));
    recipients.insert("Gap8aLe6v41t5W7hAgcL5psKYQER3GGRYn", math::precise(0.25*amount));
    recipients.insert("GUsRaKq6X5UQE8zBWsuwYBJUNxp35zDfFi", math::precise(0.1*amount));
    recipients.insert("GQ8osQUL25fgjaWnFJm6bHUYawPL53cUC5", math::precise(0.05*amount));
    let recipients_string = serde_json::to_string(&recipients)?;
    let request = format!(r#"sendmany "" {} 1"#, recipients_string);
    let raw = call(&request, auth).await?;
    let txid = serde_json::from_value::<String>(raw)?;
    Ok(txid)
}
