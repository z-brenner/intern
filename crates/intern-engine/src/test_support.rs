//! Test doubles shared by the engine's unit tests: a scripted loopback HTTP
//! server and the replies it gives.

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use serde_json::json;

/// A reply the grammar would produce for a one-line memo.
pub(crate) const VALID_REPLY: &str = r#"{"type_evidence":"MEMO","document_type":"Memo","date_evidence":null,"document_date":null,"date_role":null,"parties":[],"party_evidence":[],"party_relation":"none","description":"A memo about the office move.","confidence":0.8,"needs_review":false}"#;

/// A loopback server that answers its `n`th connection with `replies[n]`
/// (the last one again once they run out), counts the connections, and keeps
/// the body of every request it was sent.
///
/// Never joined: a test that expects no further connection leaves it
/// waiting, and the harness ends the process when the last test finishes.
pub(crate) struct ScriptedServer {
    pub(crate) address: SocketAddr,
    attempts: Arc<AtomicUsize>,
    bodies: Arc<std::sync::Mutex<Vec<String>>>,
}

impl ScriptedServer {
    pub(crate) fn attempts(&self) -> usize {
        self.attempts.load(Ordering::SeqCst)
    }

    /// The request bodies received so far, in order.
    pub(crate) fn bodies(&self) -> Vec<String> {
        self.bodies.lock().unwrap().clone()
    }
}

pub(crate) fn scripted_server(replies: Vec<Vec<u8>>) -> ScriptedServer {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let attempts = Arc::new(AtomicUsize::new(0));
    let bodies = Arc::new(std::sync::Mutex::new(Vec::new()));
    let counted = Arc::clone(&attempts);
    let kept = Arc::clone(&bodies);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { return };
            let index = counted.fetch_add(1, Ordering::SeqCst);
            // Drain the request so closing the socket cannot reset it before
            // the status line arrives.
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut length = 0_usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() || line == "\r\n" || line.is_empty() {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse().unwrap_or(0);
                }
            }
            let mut body = vec![0_u8; length];
            let _ = reader.read_exact(&mut body);
            kept.lock()
                .unwrap()
                .push(String::from_utf8_lossy(&body).into_owned());
            let reply = &replies[index.min(replies.len() - 1)];
            let _ = stream.write_all(reply);
            let _ = stream.flush();
        }
    });
    ScriptedServer {
        address,
        attempts,
        bodies,
    }
}

/// An HTTP/1.1 reply that closes its connection, so every attempt is a
/// connection the scripted server can count.
pub(crate) fn http_reply(status: &str, headers: &[(&str, &str)], body: &str) -> Vec<u8> {
    let mut reply = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (name, value) in headers {
        reply.push_str(&format!("{name}: {value}\r\n"));
    }
    reply.push_str("\r\n");
    reply.push_str(body);
    reply.into_bytes()
}

/// A chat completion whose one choice ended for `finish_reason` with
/// `content` as its text.
pub(crate) fn completion_reply(finish_reason: &str, content: &str) -> Vec<u8> {
    let body = json!({
        "choices": [{
            "finish_reason": finish_reason,
            "message": {"role": "assistant", "content": content}
        }]
    });
    http_reply(
        "200 OK",
        &[("Content-Type", "application/json")],
        &body.to_string(),
    )
}
