use std::sync::Arc;

use tokio::{io::AsyncRead, io::AsyncReadExt, io::AsyncWriteExt, net::TcpStream};

use crate::{Throw, Throws, throw};

pub enum HTTPRequest {
    Get { msg: Arc<[u8]> },
    Head { to_get: Arc<[u8]> },
    Post { to_post: Arc<[u8]> },
    Put { to_put: Arc<[u8]> },
    Delete { to_delete: Arc<[u8]> },
    Connect { to_connect_to: Arc<[u8]> },
    Options { request: Arc<[u8]> },
    Trace { request: Arc<[u8]> },
    Patch { request: Arc<[u8]> },
}

pub async fn http_write_string_response(stream: &mut TcpStream, s: &[u8]) -> Throws<()> {
    let f = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=UTF-8\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\n",
        s.len()
    );
    stream.write(f.as_bytes()).await?;
    stream.write(s).await?;
    Ok(())
}

pub async fn http_write_html_response(stream: &mut TcpStream, s: &[u8]) -> Throws<()> {
    let f = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\n",
        s.len()
    );
    stream.write(f.as_bytes()).await?;
    stream.write(s).await?;
    Ok(())
}

pub async fn http_write_png_response(stream: &mut TcpStream, s: &[u8]) -> Throws<()> {
    let f = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\ncharset=UTF-8\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\n",
        s.len()
    );
    stream.write(f.as_bytes()).await?;
    stream.write(s).await?;
    Ok(())
}

pub async fn http_write_jpg_response(stream: &mut TcpStream, s: &[u8]) -> Throws<()> {
    let f = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\ncharset=UTF-8\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\n",
        s.len()
    );
    stream.write(f.as_bytes()).await?;
    stream.write(s).await?;
    Ok(())
}

pub async fn http_write_json_response(stream: &mut TcpStream, s: &[u8]) -> Throws<()> {
    let f = format!(
        "HTTP/1.1 200 OK\r\nContent-Type:text/json\r\ncharset=UTF-8\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\n",
        s.len()
    );
    stream.write(f.as_bytes()).await?;
    stream.write(s).await?;
    Ok(())
}

pub async fn http_write_js_response(stream: &mut TcpStream, s: &[u8]) -> Throws<()> {
    let f = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/javascript; charset=UTF-8\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\n",
        s.len()
    );
    stream.write(f.as_bytes()).await?;
    stream.write(s).await?;
    Ok(())
}

pub fn get_extension(s: &str) -> &str {
    let split = s.split('.');
    let mut last = None;
    for i in split {
        last = Some(i);
    }
    let ext = if let Some(e) = last { e } else { "" };
    ext
}

#[test]
fn extension_tests() {
    assert_eq!(get_extension("test.png"), "png");
    assert_eq!(get_extension("test.bak.png"), "png");
    assert_eq!(get_extension("test.jpeg"), "jpeg");
    assert_eq!(get_extension("test.bak.jpeg"), "jpeg");
}

pub async fn read_line(stream: &mut TcpStream) -> Throws<Vec<u8>> {
    let mut idx = 0;
    let mut tbuf = [0];
    let mut buf = [0; 256];
    let mut out = Vec::new();
    loop {
        let x = stream.read(&mut tbuf).await?;
        if x == 0 {
            break;
        }
        if tbuf[0] == b'\n' {
            break;
        }
        buf[idx] = tbuf[0];
        idx += 1;
        if buf.len() <= idx {
            for i in buf {
                out.push(i);
            }
            idx = 0;
        }
    }
    for i in 0..idx {
        out.push(buf[i]);
    }
    return Ok(out);
}
pub async fn get_request(stream: &mut TcpStream) -> Throws<HTTPRequest> {
    let header = read_line(stream).await?;
    let header_string = std::str::from_utf8(&header)?;
    let mut args = header_string.split_ascii_whitespace();
    let cmd = args.next().throw()?;
    match cmd {
        "GET" => {}
        ""
        _ => {
            throw!(format!("error unknown argument to http request:{:#?}", cmd));
        }
    }
    todo!()
}

