use super::*;

#[test]
fn a_connect_names_its_host_and_port() {
    let asked = parse(b"CONNECT api.anthropic.com:443 HTTP/1.1\r\nHost: x\r\n\r\n").expect("asked");
    assert_eq!(
        asked,
        Asked {
            host: "api.anthropic.com".into(),
            port: 443,
            forward: None
        }
    );
    let v6 = parse(b"CONNECT [2606:4700::1111]:443 HTTP/1.1\r\n\r\n").expect("v6");
    assert_eq!((v6.host.as_str(), v6.port), ("2606:4700::1111", 443));
    assert!(parse(b"CONNECT example.com HTTP/1.1\r\n\r\n").is_none());
}

#[test]
fn plain_http_goes_on_with_its_path_and_without_the_proxy_headers() {
    let head = b"GET http://Example.com/a/b?c=1 HTTP/1.1\r\nHost: example.com\r\nProxy-Connection: keep-alive\r\nAccept: */*\r\n\r\n";
    let asked = parse(head).expect("asked");
    assert_eq!((asked.host.as_str(), asked.port), ("example.com", 80));
    assert_eq!(
        String::from_utf8(asked.forward.expect("forward")).expect("text"),
        "GET /a/b?c=1 HTTP/1.1\r\nHost: example.com\r\nAccept: */*\r\nConnection: close\r\n\r\n"
    );
    let port = parse(b"POST http://example.com:8080 HTTP/1.1\r\n\r\n").expect("port");
    assert_eq!(port.port, 8080);
    // A request that is not for a proxy is refused.
    assert!(parse(b"GET /mcp HTTP/1.1\r\n\r\n").is_none());
    assert!(parse(b"GET https://example.com/ HTTP/1.1\r\n\r\n").is_none());
}

/// A server on loopback that answers `hello` to whoever connects.
async fn listening() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let _ = stream.write_all(b"hello").await;
        }
    });
    port
}

async fn through_gate(gate: Arc<Gate>, port: u16) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let gate_port = listener.local_addr().expect("addr").port();
    tokio::spawn(serve_tcp(listener, gate));
    let mut client = TcpStream::connect(("127.0.0.1", gate_port))
        .await
        .expect("gate");
    let ask = format!("CONNECT 127.0.0.1:{port} HTTP/1.1\r\n\r\n");
    client.write_all(ask.as_bytes()).await.expect("ask");
    let mut said = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(5), client.read_to_end(&mut said)).await;
    String::from_utf8_lossy(&said).into_owned()
}

#[tokio::test]
async fn this_computer_is_out_of_reach() {
    let port = listening().await;
    let said = through_gate(Gate::new(&[]), port).await;
    assert!(said.starts_with("HTTP/1.1 403"), "{said}");
    assert!(!said.contains("hello"), "{said}");
}

#[tokio::test]
async fn an_address_the_owner_opened_is_reached() {
    let port = listening().await;
    let said = through_gate(Gate::new(&["127.0.0.1".to_owned()]), port).await;
    assert!(
        said.starts_with("HTTP/1.1 200 Connection established\r\n\r\nhello"),
        "{said}"
    );
}

#[tokio::test]
async fn a_name_for_this_computer_is_refused_too() {
    let port = listening().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let gate_port = listener.local_addr().expect("addr").port();
    tokio::spawn(serve_tcp(listener, Gate::new(&[])));
    let mut client = TcpStream::connect(("127.0.0.1", gate_port))
        .await
        .expect("gate");
    let ask = format!("GET http://localhost:{port}/ HTTP/1.1\r\nHost: localhost\r\n\r\n");
    client.write_all(ask.as_bytes()).await.expect("ask");
    let mut said = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(5), client.read_to_end(&mut said)).await;
    let said = String::from_utf8_lossy(&said);
    assert!(said.starts_with("HTTP/1.1 403"), "{said}");
}
