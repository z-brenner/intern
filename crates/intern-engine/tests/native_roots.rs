//! The model download and a hosted model trust the roots the operating system
//! offers, not only the Mozilla set bundled into the binary.
//!
//! A firm that inspects TLS - Zscaler, Netskope, Blue Coat - installs its own
//! root in the Windows store by policy and re-signs every connection with it.
//! With the bundled roots alone, the first-run model download failed behind
//! such a proxy with nothing saying why, and so did a hosted model.
//!
//! rustls-native-certs reads `SSL_CERT_FILE` in place of the platform store,
//! which stands in for that policy here: a throwaway CA (see
//! `fixtures/tls/README.md`) signs a local HTTPS server, and the clients must
//! reach it with the CA offered and refuse it without. The variable is
//! process-wide, and setting it in a running test binary is unsound, so each
//! case runs as a child of this binary with its own environment.

use std::{
    fs::File,
    io::{BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::Arc,
    thread,
};

use intern_engine::{
    EngineErrorCode, HostedClient, HostedModelConfig, HostedProvider, ModelClient, ModelRequest,
    Proposer,
    download::{CancellationToken, HttpTransport, ReqwestHttpTransport},
};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

/// Set for the child processes; the child tests are inert without it.
const CHILD: &str = "INTERN_NATIVE_ROOTS_CHILD";

const MODEL_BYTES: &[u8] = b"not really a model";

const REPLY: &str = r#"{"type_evidence":"MEMO","document_type":"Memo","date_evidence":null,"document_date":null,"date_role":null,"parties":[],"party_evidence":[],"party_relation":"none","description":"A memo about the office move.","confidence":0.8,"needs_review":false}"#;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tls")
        .join(name)
}

/// Runs the ignored test `name` from this binary in a child process that
/// reads its certificate store from `cert_file` (the platform's own when
/// `None`) and reaches loopback without any proxy.
fn run_child(name: &str, cert_file: Option<&Path>) -> Output {
    let mut command = Command::new(std::env::current_exe().expect("the test binary"));
    command
        .args([name, "--exact", "--ignored", "--nocapture"])
        .env(CHILD, "1")
        .env("NO_PROXY", "127.0.0.1,localhost")
        .env("no_proxy", "127.0.0.1,localhost")
        .env_remove("SSL_CERT_DIR");
    for proxy in [
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
        "ALL_PROXY",
        "all_proxy",
    ] {
        command.env_remove(proxy);
    }
    match cert_file {
        Some(cert_file) => command.env("SSL_CERT_FILE", cert_file),
        None => command.env_remove("SSL_CERT_FILE"),
    };
    command.output().expect("the child test runs")
}

fn assert_child_passed(output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("1 passed"),
        "{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn private_ca_from_ssl_cert_file_is_trusted() {
    assert_child_passed(&run_child(
        "reaches_the_private_ca_server",
        Some(&fixture("ca.pem")),
    ));
}

#[test]
fn without_it_tls_fails() {
    assert_child_passed(&run_child("cannot_reach_the_private_ca_server", None));
}

/// reqwest refuses to build a client at all when the store yields no usable
/// certificate and at least one unusable one - here a single block that is
/// not a certificate. A broken store must not stop the download, a hosted
/// model, or the local model, which never uses TLS.
#[test]
fn a_store_with_nothing_usable_in_it_does_not_block_a_client() {
    let directory = tempfile::tempdir().unwrap();
    let broken = directory.path().join("broken.pem");
    std::fs::write(
        &broken,
        "-----BEGIN CERTIFICATE-----\nbm90IGEgY2VydGlmaWNhdGU=\n-----END CERTIFICATE-----\n",
    )
    .unwrap();
    assert_child_passed(&run_child(
        "builds_every_client_over_a_broken_store",
        Some(&broken),
    ));
}

#[test]
#[ignore = "child process of private_ca_from_ssl_cert_file_is_trusted"]
fn reaches_the_private_ca_server() {
    if std::env::var_os(CHILD).is_none() {
        return;
    }
    let port = serve_tls();

    let mut response = ReqwestHttpTransport::new()
        .unwrap()
        .get(
            &format!("https://127.0.0.1:{port}/model.gguf"),
            None,
            &CancellationToken::new(),
        )
        .expect("the download reaches a server signed by the offered CA");
    assert_eq!(response.status, 200);
    let mut body = Vec::new();
    response.body.read_to_end(&mut body).unwrap();
    assert_eq!(body, MODEL_BYTES);

    let proposal = hosted(port)
        .propose(&ModelRequest {
            prompt: "File this.".into(),
        })
        .expect("the hosted client reaches a server signed by the offered CA");
    assert_eq!(proposal.document_type.as_deref(), Some("Memo"));
}

#[test]
#[ignore = "child process of without_it_tls_fails"]
fn cannot_reach_the_private_ca_server() {
    if std::env::var_os(CHILD).is_none() {
        return;
    }
    let port = serve_tls();

    let error = ReqwestHttpTransport::new()
        .unwrap()
        .get(
            &format!("https://127.0.0.1:{port}/model.gguf"),
            None,
            &CancellationToken::new(),
        )
        .err()
        .expect("a server signed by an unknown CA is refused");
    assert_eq!(error.code(), EngineErrorCode::DownloadInterrupted);

    let error = hosted(port)
        .propose(&ModelRequest {
            prompt: "File this.".into(),
        })
        .expect_err("a server signed by an unknown CA is refused");
    assert_eq!(error.code(), EngineErrorCode::HostedModelUnreachable);
}

#[test]
#[ignore = "child process of a_store_with_nothing_usable_in_it_does_not_block_a_client"]
fn builds_every_client_over_a_broken_store() {
    if std::env::var_os(CHILD).is_none() {
        return;
    }
    assert!(ReqwestHttpTransport::new().is_ok(), "the model download");
    hosted(443);
    assert!(
        ModelClient::new("http://127.0.0.1:9/v1/chat/completions", "k", "m").is_ok(),
        "the local model"
    );
}

fn hosted(port: u16) -> HostedClient {
    HostedClient::new(HostedModelConfig {
        provider: HostedProvider::OpenAiCompatible,
        base_url: format!("https://localhost:{port}/v1"),
        model: "test-model".into(),
        api_key: "sk-test".into(),
    })
    .expect("a hosted client")
}

/// Serves HTTPS on a loopback port with the fixture certificate, one reply
/// per connection: the model bytes for a GET, a chat completion for a POST.
/// Never joined; the child process ends when its one test does.
fn serve_tls() -> u16 {
    let certificates = rustls_pemfile::certs(&mut BufReader::new(
        File::open(fixture("server.pem")).unwrap(),
    ))
    .collect::<Result<Vec<CertificateDer<'static>>, _>>()
    .unwrap();
    let key: PrivateKeyDer<'static> = rustls_pemfile::private_key(&mut BufReader::new(
        File::open(fixture("server.key")).unwrap(),
    ))
    .unwrap()
    .expect("the fixture key");
    // The provider is named rather than left to the process default, which
    // is ambiguous whenever a build enables more than one.
    let config = Arc::new(
        rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(certificates, key)
        .unwrap(),
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { return };
            let config = Arc::clone(&config);
            thread::spawn(move || answer(stream, config));
        }
    });
    port
}

/// One request and its reply over TLS. A client that refuses the
/// certificate ends the handshake, and with it this connection, quietly.
fn answer(mut socket: TcpStream, config: Arc<rustls::ServerConfig>) {
    let Ok(mut connection) = rustls::ServerConnection::new(config) else {
        return;
    };
    {
        let mut tls = rustls::Stream::new(&mut connection, &mut socket);
        let mut head = Vec::new();
        let mut byte = [0_u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            match tls.read(&mut byte) {
                Ok(1) => head.push(byte[0]),
                _ => return,
            }
        }
        let head = String::from_utf8_lossy(&head).to_ascii_lowercase();
        let length = head
            .lines()
            .find_map(|line| line.strip_prefix("content-length:"))
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        let mut body = vec![0_u8; length];
        if tls.read_exact(&mut body).is_err() {
            return;
        }
        let (content_type, reply) = if head.starts_with("get ") {
            ("application/octet-stream", MODEL_BYTES.to_vec())
        } else {
            let completion = serde_json::json!({
                "choices": [{
                    "finish_reason": "stop",
                    "message": {"role": "assistant", "content": REPLY}
                }]
            });
            ("application/json", completion.to_string().into_bytes())
        };
        let mut response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            reply.len()
        )
        .into_bytes();
        response.extend_from_slice(&reply);
        if tls.write_all(&response).and_then(|()| tls.flush()).is_err() {
            return;
        }
    }
    connection.send_close_notify();
    let _ = connection.complete_io(&mut socket);
}
