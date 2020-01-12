mod net;
mod snake;

use log::*;
use serde_json::json;

extern crate env_logger;
extern crate log;
extern crate serde;
extern crate serde_json;

fn handle_move(board: serde_json::value::Value) -> Result<serde_json::value::Value, u16> {
  Ok(json!({"direction": "up"}))
}

fn handle(mut ctx: net::Context) {
  let url = match ctx.read_request() {
    Ok(url) => url,
    Err(e) => {
      error!("http: failed to read headers: {}", e);
      return;
    }
  };
  let r: Result<serde_json::value::Value, u16> = match url.as_str() {
    "/move" => match ctx.read_json() {
      Ok(val) => handle_move(val),
      Err(e) => {
        error!("http: failed to read body {}", e);
        Err(400)
      }
    },
    "/start" => Ok(json!({"ok": true})),
    "/end" => Ok(json!({"ok": true})),
    "/ping" => Ok(json!({"ok": true})),
    _ => Err(404),
  };

  let code: u16;
  let value: serde_json::value::Value;
  match r {
    Ok(j) => {
      code = 200;
      value = j;
    }
    Err(c) => {
      code = c;
      value = json!({ "code": c });
    }
  };
  match ctx.write_status(code) {
    Ok(_) => (),
    Err(e) => {
      error!("http: could not write status {}", e);
      return;
    }
  };
  match ctx.write_json(&value) {
    Ok(_) => (),
    Err(e) => {
      error!("http: could not write body {}", e);
      return;
    }
  };
}

fn main() {
  env_logger::init();
  net::server("127.0.0.1:3000", handle);
}
