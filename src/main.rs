mod net;

extern crate env_logger;
extern crate log;
extern crate serde;
extern crate serde_json;

use serde_json::json;

fn handle(mut ctx: net::Context) {
  match ctx.read_body() {
    Err(_) => return,
    _ => (),
  };
  let value = json!({
      "code": 200,
  });
  match ctx.write_status(200) {
    Err(e) => {
      println!("could not read {}", e);
    },
    Ok(_) => (),
  }
  match ctx.write_json(&value) {
    Err(e) => {
      println!("could not write {}", e);
    },
    Ok(_) => (),
  }
}

fn main() {
  env_logger::init();
  net::server("127.0.0.1:3000", handle);
}
