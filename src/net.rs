use log::*;
use serde::{Serialize};
use serde::de::{DeserializeOwned};
use serde_json;
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Result, Write};
use std::net::{TcpListener, TcpStream};

const PRE_HTTP: &[u8] = "HTTP/1.1 ".as_bytes();
const POST_HTTP: &[u8] = " OK\r\n".as_bytes();
const SEP: &[u8] = "\r\n".as_bytes();
const SPACE: &[u8] = " ".as_bytes();

#[derive(Clone, Debug)]
pub enum Method {
  Get,
  Post,
  Patch,
  Put,
  Delete,
}

#[derive(Clone, Debug)]
pub struct Request {
  pub method: Method,
  pub url: String,
  pub headers: HashMap<String, String>,
  pub size: usize,
}

impl Request {
  pub fn new() -> Request {
    Request{
      method: Method::Get,
      url: String::new(),
      headers: HashMap::new(),
      size: 0,
    }
  }
}

pub struct Context {
  stream: TcpStream,
  request: Option<Request>,
  reader: BufReader<TcpStream>,
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

  pub fn read_json<T>(&mut self) -> Result<T>
  where
    T: DeserializeOwned,
  {
    let r = &mut self.reader;
    let req = match &self.request {
      Some(r) => r,
      None => return Result::Err(Error::new(ErrorKind::Other, "no request")),
    };
    match serde_json::from_reader(r.take(req.size.try_into().unwrap())) {
      Ok(r) => Result::Ok(r),
      Err(e) => Result::Err(Error::new(ErrorKind::Other, e.to_string())),
    }
  }

  #[inline]
  fn read_first_line(s: String) -> Result<(Method, String)> {
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

  #[inline]
  fn read_header_line(s: String) -> Result<(String, String)> {
    let mut line = s.split_whitespace();
    let name = match line.next() {
      Some(h) => {
        let mut s = h.to_lowercase();
        s.pop();
        s
      },
      None => return Result::Err(Error::new(ErrorKind::Other, "no header name")),
    };
    let value = match line.next() {
      Some(h) => h,
      None => return Result::Err(Error::new(ErrorKind::Other, "no header value")),
    };
    return Result::Ok((String::from(name), String::from(value)));
  }

  #[inline]
  pub fn read_request(&mut self) -> Result<String> {
    let r = &mut self.reader;
    let mut req = Request::new();
    let mut url: String = String::new();
    let mut first = true;

    for line in r.lines() {
      match line {
        Ok(s) => {
          if s.len() == 0 {
            break;
          }

          if first {
            match Context::read_first_line(s) {
              Ok((m, u)) => {
                req.method = m;
                req.url = u.clone();
                url = u;
              }
              Err(e) => return Result::Err(e),
            };
            first = false;
          } else {
            match Context::read_header_line(s) {
              Ok((k, v)) => {
                req.headers.insert(k, v);
              }
              Err(e) => return Result::Err(e),
            }
          }
        }
        Err(e) => return Result::Err(e),
      }
    }

    req.size = match req.headers.get("content-length") {
      Some(len) => len.parse().unwrap(),
      None => 0,
    };

    self.request = Option::Some(req);
    return Result::Ok(url);
  }

  pub fn write_status(&mut self, i: u16) -> Result<()> {
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
        let reader = BufReader::new(stream.try_clone().unwrap());
        h(Context { stream, reader, request: Option::None });
      }
      Err(e) => error!("http: connection failed {}", e),
    }
  }
}
