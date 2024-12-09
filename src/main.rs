use base64;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::io::{self, AsyncBufReadExt}; // For reading input from stdin
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
struct IncomingMessage {
    //sent_at: String,
    sender: String,
    receiver: String,
    message_type: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ResponseMessage {
    response_type: String,
    content: String,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let USERNAME = args[1].clone(); // Clone the username
                                    // Connect to the WebSocket server
                                    // let username = "user";
                                    // let password = "password";
                                    // let credentials = format!("{}:{}", username, password);
    let PASSWORD = args[2].clone();
    let encoded_password = STANDARD.encode(PASSWORD);
    let url_str = format!(
        "ws://127.0.0.1:3030/ws?name={}&password={}",
        &USERNAME, &encoded_password
    );
    let url = Url::parse(&url_str).unwrap();
    // Create the WebSocket request with the Authorization header
    let mut request = url.into_client_request().expect("Invalid WebSocket URL");
    request.headers_mut().insert(
        "Authorization",
        // format!("{}", credentials).parse().unwrap(),
        format!("{}", &USERNAME).parse().unwrap(),
    );

    let (ws_stream, _) = connect_async(request)
        .await
        .expect("Failed to connect to WebSocket server");

    let (mut write, mut read) = ws_stream.split();

    // Task to handle receiving messages from the server
    let receive_task = tokio::spawn(async move {
        while true {
            if let Some(Ok(message)) = read.next().await {
                if let Message::Text(text) = message {
                    // Deserialize the response JSON
                    if let Ok(response) = serde_json::from_str::<IncomingMessage>(&text) {
                        println!("message from {:?}: {:?}", response.sender, response.content);
                    }
                }
            }
        }
    });

    // Task to handle user input and send messages to the server
    let send_task = tokio::spawn(async move {
        let stdin = io::BufReader::new(io::stdin());
        let mut lines = stdin.lines();

        while let Ok(line) = lines.next_line().await {
            if line.is_none() {
                continue;
            }

            let line = line.unwrap();
            let (receiver_username, message) = line.split_once(char::is_whitespace).unwrap();
            //let timestamp: DateTime<Utc> = Utc::now();

            let outgoing_message = IncomingMessage {
                //sent_at: timestamp.to_rfc3339(),
                sender: USERNAME.clone(),
                receiver: receiver_username.to_string(),
                message_type: "text".to_string(),
                content: message.to_string(), // Send the user's input as the content
            };

            let serialized_msg = serde_json::to_string(&outgoing_message).unwrap();
            if let Err(e) = write.send(Message::Text(serialized_msg)).await {
                println!("Failed to send message: {:?}", e);
                break;
            }
        }
    });

    // Wait for both tasks to complete
    let _ = tokio::join!(receive_task, send_task);
}
