use log::*;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Result, Write};
use std::net::{TcpListener, TcpStream};

const PRE_HTTP: &[u8] = "HTTP/1.1 ".as_bytes();
const POST_HTTP: &[u8] = " OK\r\n".as_bytes();
const SEP: &[u8] = "\r\n".as_bytes();
const SPACE: &[u8] = " ".as_bytes();

pub enum Method {
  Get,
  Post,
  Patch,
  Put,
  Delete,
}

pub struct Context {
  stream: TcpStream,
}

impl Context {
  pub fn write_json<T: ?Sized>(&mut self, body: &T) -> Result<()>
  where
    T: Serialize,
  {
    let buf = vec![0; 64];
    let mut w = BufWriter::new(buf);

    match serde_json::to_writer(&mut w, body) {
      Err(e) => return Result::Err(Error::new(ErrorKind::Other, e)),
      _ => (),
    };
    self.write_body("application/json", w.buffer())
  }

  pub fn read_body(&mut self) -> Result<(Method, String, HashMap<String, String>)> {
    match self.read_body_internal() {
      Ok(b) => Result::Ok(b),
      Err(e) => {
        error!("http: body read failed {}", e);
        return Result::Err(e);
      }
    }
  }

  fn read_first_line(&mut self, s: String) -> Result<(Method, String)> {
    let mut line = s.split_whitespace();
    let method = match line.next() {
      Some(m) => match m {
        "GET" => Method::Get,
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "PATCH" => Method::Patch,
        "DELETE" => Method::Delete,
        _ => return Result::Err(Error::new(ErrorKind::Other, m)),
      },
      None => return Result::Err(Error::new(ErrorKind::Other, "missing method")),
    };
    let url = match line.next() {
      Some(u) => u,
      None => return Result::Err(Error::new(ErrorKind::Other, "missing url")),
    };
    return Result::Ok((method, String::from(url)));
  }

  fn read_header_line(&mut self, s: String) -> Result<(String, String)> {
    let mut line = s.split_whitespace();
    let name = match line.next() {
      Some(h) => h.to_lowercase(),
      None => return Result::Err(Error::new(ErrorKind::Other, "no header name")),
    };
    let value = match line.next() {
      Some(h) => h,
      None => return Result::Err(Error::new(ErrorKind::Other, "no header value")),
    };
    return Result::Ok((String::from(name), String::from(value)));
  }

  fn read_body_internal(&mut self) -> Result<(Method, String, HashMap<String, String>)> {
    let r = &mut BufReader::new(self.stream.try_clone().unwrap());
    let mut first = true;
    let mut method = Method::Get;
    let mut url = String::new();
    let mut head: HashMap<String, String> = HashMap::new();

    for line in r.lines() {
      match line {
        Ok(s) => {
          if s.len() == 0 {
            break;
          }

          if first {
            match self.read_first_line(s) {
              Ok((m, u)) => {
                method = m;
                url = u;
              }
              Err(e) => return Result::Err(e),
            };
            first = false;
          } else {
            match self.read_header_line(s) {
              Ok((k, v)) => {
                head.insert(k, v);
              }
              Err(e) => return Result::Err(e),
            }
          }
        }
        Err(e) => return Result::Err(e),
      }
    }

    let size: usize = match head.get("content-length") {
      Some(len) => len.parse().unwrap(),
      None => 0,
    };
    let mut buf = vec![0; size];
    match r.take(size.try_into().unwrap()).read(&mut buf) {
      Err(e) => return Result::Err(e),
      _ => (),
    };
    return Result::Ok((method, url, head));
  }

  pub fn write_status(&mut self, i: u8) -> Result<()> {
    let s: String = i.to_string();
    self
      .stream
      .write_all(PRE_HTTP)
      .and_then(|_| self.stream.write_all(s.as_bytes()))
      .and_then(|_| self.stream.write_all(POST_HTTP))
  }

  pub fn write_header(&mut self, header: &str, val: &str) -> Result<()> {
    self
      .stream
      .write_all(header.as_bytes())
      .and_then(|_| self.stream.write_all(SPACE))
      .and_then(|_| self.stream.write_all(val.as_bytes()))
      .and_then(|_| self.stream.write_all(SEP))
  }

  pub fn write_body(&mut self, content_type: &str, body: &[u8]) -> Result<()> {
    let len: String = body.len().to_string();
    self
      .write_header("content-type", content_type)
      .and_then(|_| self.write_header("content-length", len.as_str()))
      .and_then(|_| self.stream.write_all(SEP))
      .and_then(|_| self.stream.write_all(body))
      .and_then(|_| self.stream.flush())
  }
}

type Handler = fn(ctx: Context);

pub fn server(s: &'static str, h: Handler) {
  let listener = TcpListener::bind(s).unwrap();
  info!("listener started on {}", s);
  for stream in listener.incoming() {
    match stream {
      Ok(stream) => {
        h(Context { stream });
        info!("http: handled request");
      }
      Err(e) => error!("http: connection failed {}", e),
    }
  }
}
