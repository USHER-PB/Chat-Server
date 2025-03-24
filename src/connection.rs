use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};

pub(crate) fn connection(mut stream: TcpStream, messages: Arc<Mutex<Vec<String>>>) {
    let mut buffer = [0; 1024];

    // Read request from stream
    let bytes_read = match stream.read(&mut buffer) {
        Ok(0) => {
            eprintln!("Client disconnected.");
            return;
        }
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to read from stream: {}", e);
            return;
        }
    };

    // Convert request to string
    let request = match std::str::from_utf8(&buffer[..bytes_read]) {
        Ok(req) => req.trim(),
        Err(e) => {
            eprintln!("Invalid UTF-8 request: {}", e);
            return;
        }
    };

    // Print full request for debugging
    println!("Received request:\n{}", request);

    // Split request into lines
    let mut request_lines = request.lines();

    if let Some(first_line) = request_lines.next() {
        let request_parts: Vec<&str> = first_line.trim().split_whitespace().collect();

        if request_parts.len() < 2 {
            eprintln!("Malformed request.");
            return;
        }

        // Handle GET /send
        if request_parts[0] == "GET" && request_parts[1] == "/send" {
            let messages_guard = match messages.lock() {
                Ok(messages) => messages,
                Err(e) => {
                    eprintln!("Failed to lock messages: {}", e);
                    let response = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes());
                    return;
                }
            };

            let response_body = format!("{:?}", *messages_guard);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                response_body.len(),
                response_body
            );

            if let Err(e) = stream.write_all(response.as_bytes()) {
                eprintln!("Failed to send response: {}", e);
            }
        }
        // Handle POST /retrieve
        else if request_parts[0] == "POST" && request_parts[1] == "/retrieve" {
            // Collect remaining lines as body
            let body: String = request_lines.collect::<Vec<&str>>().join("\n");

            let mut messages_guard = match messages.lock() {
                Ok(messages) => messages,
                Err(e) => {
                    eprintln!("Failed to lock messages: {}", e);
                    let response = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes());
                    return;
                }
            };

            // Store the received message
            messages_guard.push(body.clone());

            let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK";
            if let Err(e) = stream.write_all(response.as_bytes()) {
                eprintln!("Failed to send response: {}", e);
            }
        } else {
            eprintln!("Unknown request.");
            let response = "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes());
        }
    }
}
