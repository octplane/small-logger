use std::path::Path;
use std::error::Error;

use staticfile::Static;

use iron::prelude::*;
use iron::status;
use iron;
use mount::Mount;
use router::{Router};
use api;

use rustc_serialize::json;

#[derive(RustcEncodable)]
struct LogFileList {
  files: Vec<String>
}

#[derive(RustcEncodable)]
struct ErrorMessage {
  message: String
}

fn handler(req: &mut Request) -> IronResult<Response> {
  let query = req.extensions.get::<Router>().unwrap().find("method").unwrap_or("/");

  let response = if query == "list" {
    let payload = match api::find_files("./logs") {
      Ok(logs) => json::encode(&LogFileList{ files: logs.iter().map(|&ref file| file[1..].to_string()).collect() }).unwrap(),
      Err(e) => json::encode(&ErrorMessage{message: e.description().to_string()}).unwrap()
    };

    let mut response = Response::with((status::Ok, payload));
    let jsony_ctype = iron::headers::ContentType(iron::mime::Mime(
      iron::mime::TopLevel::Application,
      iron::mime::SubLevel::Json,
      vec![(iron::mime::Attr::Charset, iron::mime::Value::Utf8)]));

    response.headers.set::<iron::headers::ContentType>(jsony_ctype);
    response
  } else {
    Response::with(status::Ok)
  };

  Ok(response)
}

pub fn startup() {
  println!("Starting HTTP Daemon...");

  match api::find_files("./logs") {
    Ok(logs) => println!("Files: {:?}", logs),
    Err(e) => println!("Error: {:?}", e)
  }

  // FileMeta::fast_meta("./logs/2015/03/24/ls-17:30:30.ajson".to_string());

  let mut mount = Mount::new();

  mount.mount("/api/1/files/logs", Static::new(Path::new("logs")));
  mount.mount("/viewer", Static::new(Path::new("viewer")));

  let mut router = Router::new();
  router.get("/:method", handler);
  mount.mount("/api/1/", router);

  println!("Open http://localhost:5001/viewer/");
  Iron::new(mount).http("0.0.0.0:5001").unwrap();
}
