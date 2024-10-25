use std::env;
use tokio_tungstenite::connect_async;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use serde::{Serialize, Deserialize};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use url::Url;
use tokio::io::{self, AsyncBufReadExt}; // For reading input from stdin

#[derive(Serialize, Deserialize, Debug)]
struct IncomingMessage {
    from_id: String,
    to_id: String,
    message_type: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ResponseMessage {
    response_type: String,
    content: String,
}

static mut USERNAME: String = String::new();

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let username = args[1].clone(); // Clone the username
    // Connect to the WebSocket server
    // let username = "user";
    // let password = "password";
    // let credentials = format!("{}:{}", username, password);
    let url = Url::parse("ws://127.0.0.1:3030/ws").unwrap();
    // Create the WebSocket request with the Authorization header
    let mut request = url.into_client_request().expect("Invalid WebSocket URL");
    request.headers_mut().insert(
        "Authorization",
        // format!("{}", credentials).parse().unwrap(),
        format!("{}", username).parse().unwrap()
    );
    let (ws_stream, _) = connect_async(request).await.expect("Failed to connect to WebSocket server");

    let (mut write, mut read) = ws_stream.split();

    // Send a JSON message to the server
    // let outgoing_message = IncomingMessage {
    //     from_id: username.to_string(),
    //     to_id: username.to_string(),
    //     message_type: "text".to_string(),
    //     content: "Hello, server!".to_string(),
    // };

    // let serialized_msg = serde_json::to_string(&outgoing_message).unwrap();
    // write.send(Message::Text(serialized_msg)).await.expect("Failed to send message");
    //
    // let serialized_msg = serde_json::to_string(&outgoing_message).unwrap();
    // write.send(Message::Text(serialized_msg)).await.expect("Failed to send message");

    // Wait for a response from the server
    // Task to handle receiving messages from the server
    let receive_task = tokio::spawn(async move {
        while true {
            if let Some(Ok(message)) = read.next().await {
                if let Message::Text(text) = message {
                    // Deserialize the response JSON
                    if let Ok(response) = serde_json::from_str::<ResponseMessage>(&text) {
                        println!("Received from server: {:?}", response);

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
            if line.is_none() {continue;}
            let outgoing_message = IncomingMessage {
                from_id: username.to_string(),
                to_id: username.to_string(),
                message_type: "text".to_string(),
                content: line.unwrap().clone(), // Send the user's input as the content
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
