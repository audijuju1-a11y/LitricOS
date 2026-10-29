//! A bounded HTTP/1.1 reader for pages served on this machine.
//!
//! The web engine is Chromium over the broker, and this image has no Chromium.
//! What the window *can* do without one is open a page served by a web server
//! running on this machine: a real socket, a real `GET`, a real response. That
//! is the path this module implements, and it is deliberately narrow:
//!
//! * loopback hosts only — `localhost`, `127.0.0.0/8` and `::1`;
//! * `GET` only, plain HTTP only, no TLS (this project ships no TLS stack);
//! * a response is capped by [`MAX_BODY_BYTES`], its headers by
//!   [`MAX_HEADER_BYTES`], and a body larger than the cap is refused before it
//!   is read;
//! * a chunked response is refused by name: this reader has no chunk decoder,
//!   and pretending a truncated body was the page would be worse than saying so.
//!
//! Nothing here caches, polls or keeps a connection open. One call is one
//! request and one closed socket.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::url::{Scheme, Url};

/// The largest body one page fetch may return: the same bound the HTML reader
/// accepts, so a response cannot make the parser's own limit meaningless.
pub const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;

/// The largest response head this reader will buffer.
pub const MAX_HEADER_BYTES: usize = 16 * 1024;

/// How long a connection attempt may take.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

/// How long one read may block.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// How long one write may block.
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

/// One response, already bounded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    /// The status code the server answered with.
    pub status: u16,
    /// The `Content-Type`, lowercased, when the server sent one.
    pub content_type: Option<String>,
    /// The body, never longer than the limit the call was given.
    pub body: Vec<u8>,
}

/// Whether a host names this machine.
///
/// Exactly the loopback names: `localhost`, `::1`, and `127.0.0.0/8`. A name
/// that merely *contains* `localhost` is a different machine, and a private
/// address such as `192.168.1.10` is the local network, not this machine.
#[must_use]
pub fn is_loopback_host(host: &str) -> bool {
    let host = host.trim();
    let host = host
        .strip_prefix('[')
        .and_then(|h| h.strip_suffix(']'))
        .unwrap_or(host);
    if host.eq_ignore_ascii_case("localhost") || host == "::1" {
        return true;
    }
    let mut parts = host.split('.');
    let mut count = 0usize;
    let mut first = true;
    for part in &mut parts {
        count = count.saturating_add(1);
        if part.is_empty() || part.len() > 3 || !part.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
        let Ok(value) = part.parse::<u16>() else {
            return false;
        };
        if value > 255 {
            return false;
        }
        if first {
            if value != 127 {
                return false;
            }
            first = false;
        }
    }
    count == 4
}

/// Fetch a page from a server on this machine.
///
/// # Errors
/// A sentence for the user: the address is not one this reader may open, the
/// server could not be reached, or its answer could not be read as it was sent.
pub fn get(url: &Url) -> Result<Response, String> {
    get_with_limit(url, MAX_BODY_BYTES)
}

/// Fetch an object from a server on this machine with an explicit body cap.
///
/// # Errors
/// As [`get`].
pub fn get_with_limit(url: &Url, limit: usize) -> Result<Response, String> {
    if url.scheme != Scheme::Http {
        return Err("this page is not served over plain HTTP on this machine".to_owned());
    }
    if !is_loopback_host(&url.host) {
        return Err("this page is not on this machine".to_owned());
    }
    let host = url
        .host
        .strip_prefix('[')
        .and_then(|h| h.strip_suffix(']'))
        .unwrap_or(&url.host);
    let port = url.port.unwrap_or(80);
    let mut stream = connect(host, port)?;
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));

    let authority = if url.port.unwrap_or(80) == 80 {
        host.to_owned()
    } else {
        format!("{host}:{port}")
    };
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {authority}\r\n\
         Accept: text/html,image/jpeg;q=0.9,*/*;q=0.1\r\n\
         User-Agent: LitricBrowser/1\r\nConnection: close\r\n\r\n",
        request_target(url)
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|_| "the request could not be sent".to_owned())?;

    let (head, rest) = read_head(&mut stream)?;
    let head = String::from_utf8_lossy(&head);
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or_default();
    let mut parts = status_line.split_whitespace();
    let version = parts.next().unwrap_or_default();
    if !version.starts_with("HTTP/1.") {
        return Err("the server did not answer with HTTP".to_owned());
    }
    let status = parts
        .next()
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| "the server sent no status".to_owned())?;

    let mut content_type = None;
    let mut content_length = None;
    let mut chunked = false;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim();
        match name.as_str() {
            "content-length" => content_length = value.parse::<usize>().ok(),
            "content-type" => content_type = Some(value.to_ascii_lowercase()),
            "transfer-encoding" => chunked = !value.eq_ignore_ascii_case("identity"),
            _ => {}
        }
    }
    if chunked {
        return Err(
            "the server sent a chunked response, which this page reader cannot decode".to_owned(),
        );
    }

    let body = match content_length {
        Some(n) if n > limit => Err("the page is too large to open".to_owned()),
        Some(n) => read_exactly(&mut stream, rest, n),
        None => read_to_end(&mut stream, rest, limit),
    }?;

    Ok(Response {
        status,
        content_type,
        body,
    })
}

/// Connect to the first address that answers, within the connect timeout.
fn connect(host: &str, port: u16) -> Result<TcpStream, String> {
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|_| "the server could not be reached".to_owned())?;
    for addr in addrs {
        if let Ok(stream) = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT) {
            return Ok(stream);
        }
    }
    Err("the server could not be reached".to_owned())
}

/// Read the response head, returning it and any body bytes read with it.
fn read_head(stream: &mut TcpStream) -> Result<(Vec<u8>, Vec<u8>), String> {
    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        let n = stream
            .read(&mut chunk)
            .map_err(|_| "the server did not answer".to_owned())?;
        if n == 0 {
            return Err("the server did not answer".to_owned());
        }
        let got = chunk.get(..n).unwrap_or(&[]);
        buf.extend_from_slice(got);
        if let Some(split) = find_head_end(&buf) {
            let head = buf.get(..split).unwrap_or(&[]).to_vec();
            let rest = buf.get(split..).unwrap_or(&[]).to_vec();
            return Ok((head, rest));
        }
        if buf.len() > MAX_HEADER_BYTES {
            return Err("the server's answer was too long to read".to_owned());
        }
    }
}

/// The index just past the blank line that ends a head.
fn find_head_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4)
}

/// Read exactly `n` body bytes, using the bytes already read with the head.
fn read_exactly(stream: &mut TcpStream, rest: Vec<u8>, n: usize) -> Result<Vec<u8>, String> {
    let mut body = rest;
    if body.len() > n {
        body.truncate(n);
        return Ok(body);
    }
    let mut chunk = [0u8; 8192];
    while body.len() < n {
        let read = stream
            .read(&mut chunk)
            .map_err(|_| "the server stopped before the page was complete".to_owned())?;
        if read == 0 {
            return Err("the server stopped before the page was complete".to_owned());
        }
        let take = read.min(n.saturating_sub(body.len()));
        body.extend_from_slice(chunk.get(..take).unwrap_or(&[]));
    }
    Ok(body)
}

/// Read a close-delimited body, refusing it as soon as it passes `limit`.
fn read_to_end(stream: &mut TcpStream, rest: Vec<u8>, limit: usize) -> Result<Vec<u8>, String> {
    let mut body = rest;
    if body.len() > limit {
        return Err("the page is too large to open".to_owned());
    }
    let mut chunk = [0u8; 8192];
    loop {
        let n = stream
            .read(&mut chunk)
            .map_err(|_| "the server stopped before the page was complete".to_owned())?;
        if n == 0 {
            return Ok(body);
        }
        if body.len().saturating_add(n) > limit {
            return Err("the page is too large to open".to_owned());
        }
        body.extend_from_slice(chunk.get(..n).unwrap_or(&[]));
    }
}

/// The path and query of a request target, percent-encoded where a request
/// line cannot carry a byte literally.
fn request_target(url: &Url) -> String {
    let path = if url.path.is_empty() {
        "/"
    } else {
        url.path.as_str()
    };
    let mut out = String::with_capacity(path.len().saturating_add(8));
    for b in path.bytes() {
        push_target_byte(&mut out, b);
    }
    if let Some(query) = &url.query {
        out.push('?');
        for b in query.bytes() {
            push_target_byte(&mut out, b);
        }
    }
    out
}

/// Append one byte of a request target, escaping what a request line cannot hold.
fn push_target_byte(out: &mut String, b: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let literal = b.is_ascii_alphanumeric()
        || matches!(
            b,
            b'-' | b'.'
                | b'_'
                | b'~'
                | b'/'
                | b'?'
                | b'='
                | b'&'
                | b'%'
                | b':'
                | b'@'
                | b'+'
                | b','
                | b';'
                | b'!'
                | b'$'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'['
                | b']'
        );
    if literal {
        out.push(char::from(b));
    } else {
        out.push('%');
        out.push(char::from(
            HEX.get(usize::from(b >> 4)).copied().unwrap_or(b'0'),
        ));
        out.push(char::from(
            HEX.get(usize::from(b & 0x0f)).copied().unwrap_or(b'0'),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::sync::mpsc::{self, Receiver};

    /// One real server on a real loopback socket, answering one connection with
    /// the bytes it was given and reporting the request it received. It is not
    /// a mock: the client connects, sends and reads over TCP.
    fn serve_once(reply: Vec<u8>) -> (u16, Receiver<String>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("a loopback listener");
        let port = listener.local_addr().expect("an address").port();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buf = [0u8; 2048];
            let n = stream.read(&mut buf).unwrap_or(0);
            let _ = tx.send(String::from_utf8_lossy(buf.get(..n).unwrap_or(&[])).into_owned());
            let _ = stream.write_all(&reply);
        });
        (port, rx)
    }

    fn url(port: u16, path: &str) -> Url {
        Url::parse(&format!("http://127.0.0.1:{port}{path}")).expect("a loopback url")
    }

    #[test]
    fn a_page_is_fetched_over_a_real_loopback_socket() {
        let (port, seen) = serve_once(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 11\r\n\r\nhello world"
                .to_vec(),
        );
        let response = get(&url(port, "/index.html?q=1")).expect("a real answer");
        assert_eq!(response.status, 200);
        assert_eq!(response.content_type.as_deref(), Some("text/html"));
        assert_eq!(response.body, b"hello world");
        let request = seen
            .recv_timeout(Duration::from_secs(5))
            .unwrap_or_default();
        assert!(
            request.starts_with("GET /index.html?q=1 HTTP/1.1\r\n"),
            "{request}"
        );
        assert!(request.contains("Host: 127.0.0.1:"), "{request}");
        assert!(request.contains("Connection: close"), "{request}");
    }

    #[test]
    fn a_page_with_no_length_is_read_to_the_close() {
        let (port, _seen) =
            serve_once(b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\nserved".to_vec());
        assert_eq!(get(&url(port, "/")).expect("an answer").body, b"served");
    }

    #[test]
    fn a_server_error_is_reported_as_its_status_without_pretending_it_loaded() {
        let (port, _seen) =
            serve_once(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_vec());
        let response = get(&url(port, "/missing")).expect("an answer");
        assert_eq!(response.status, 404);
        assert!(response.body.is_empty());
    }

    #[test]
    fn a_chunked_reply_is_refused_by_name() {
        let (port, _seen) = serve_once(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n0\r\n\r\n"
                .to_vec(),
        );
        let err = get(&url(port, "/")).expect_err("chunked is not decoded");
        assert!(err.contains("chunked"), "{err}");
    }

    #[test]
    fn an_oversized_content_length_is_refused_before_the_body_is_read() {
        let (port, _seen) = serve_once(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
                MAX_BODY_BYTES + 1
            )
            .into_bytes(),
        );
        let err = get(&url(port, "/")).expect_err("too large");
        assert!(err.contains("too large"), "{err}");
    }

    #[test]
    fn a_short_body_is_an_error_not_a_smaller_page() {
        let (port, _seen) =
            serve_once(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\nshort".to_vec());
        let err = get(&url(port, "/")).expect_err("incomplete");
        assert!(err.contains("before the page was complete"), "{err}");
    }

    #[test]
    fn a_refused_connection_is_a_sentence_not_a_panic() {
        // Bind and drop: the port is known to have been closed again, so the
        // connect is refused by the kernel rather than answered by anything.
        let port = {
            let listener = TcpListener::bind(("127.0.0.1", 0)).expect("a port");
            listener.local_addr().expect("an address").port()
        };
        let err = get(&url(port, "/")).expect_err("nothing is listening");
        assert_eq!(err, "the server could not be reached");
    }

    #[test]
    fn only_this_machine_is_reachable_and_only_over_plain_http() {
        let remote = Url::parse("http://example.com/").expect("a url");
        assert_eq!(
            get(&remote).expect_err("remote http"),
            "this page is not on this machine"
        );
        let tls = Url::parse("https://127.0.0.1/").expect("a url");
        assert!(get(&tls).expect_err("no tls").contains("plain HTTP"));
    }

    #[test]
    fn the_loopback_predicate_is_narrow() {
        for yes in [
            "localhost",
            "LOCALHOST",
            "::1",
            "[::1]",
            "127.0.0.1",
            "127.9.8.7",
        ] {
            assert!(is_loopback_host(yes), "{yes} should be loopback");
        }
        for no in [
            "localhost.example.com",
            "127.0.0.1.example.com",
            "128.0.0.1",
            "10.0.0.1",
            "192.168.1.10",
            "127.0.0",
            "127.0.0.256",
            "",
        ] {
            assert!(!is_loopback_host(no), "{no} should not be loopback");
        }
    }

    #[test]
    fn a_target_is_percent_encoded_where_a_request_line_cannot_hold_it() {
        let u = Url::parse("http://localhost/a b/c?q=x y").expect("a url");
        assert_eq!(request_target(&u), "/a%20b/c?q=x%20y");
        let u = Url::parse("http://localhost/").expect("a url");
        assert_eq!(request_target(&u), "/");
    }
}
