use std::path::Path;
use std::error::Error;

use staticfile::Static;

use iron::prelude::*;
use iron::status;
use urlencoded::UrlEncodedBody;

use iron;
use mount::Mount;
use router::{Router};
use api;

use rustc_serialize::json;
// use frank_jwt::Header;
// use frank_jwt::Payload;
// use frank_jwt::encode;
// use frank_jwt::decode;
// use frank_jwt::Algorithm;

#[derive(RustcEncodable)]
struct LogFileList {
  files: Vec<String>
}

#[derive(RustcEncodable)]
struct ErrorMessage {
  message: String
}

fn send_json(pair: (iron::status::Status,String)) -> IronResult<Response> {
  let mut response = Response::with(pair);

  let jsony_ctype = iron::headers::ContentType(iron::mime::Mime(
    iron::mime::TopLevel::Application,
    iron::mime::SubLevel::Json,
    vec![(iron::mime::Attr::Charset, iron::mime::Value::Utf8)]));

  response.headers.set::<iron::headers::ContentType>(jsony_ctype);
  Ok(response)
}

fn list_files() -> (iron::status::Status, String) {
  match api::find_files("./logs") {
    Ok(logs) => (status::Ok, json::encode(&LogFileList{ files: logs.iter().map(|&ref file| file[1..].to_string()).collect() }).unwrap()),
    Err(e) => (status::InternalServerError, json::encode(&ErrorMessage{message: e.description().to_string()}).unwrap())
  }
}


fn login_handler(req: &mut Request) -> IronResult<Response> {
  match req.get_ref::<UrlEncodedBody>() {
         Ok(ref hashmap) => println!("Parsed POST request query string:\n {:?}", hashmap),
         Err(ref e) => println!("{:?}", e)
     };
  Ok(Response::with((status::Ok, String::from("coucou"))))
}

fn get_handler(req: &mut Request) -> IronResult<Response> {
  let query = req.extensions.get::<Router>().unwrap().find("method").unwrap_or("/");

  if query == "list" {
    send_json(list_files())
  } else {
    Ok(Response::with(status::NotFound))
  }
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
  let mut router = Router::new();
  router.get("/:method", get_handler);
  router.post("/login", login_handler);
  mount.mount("/api/1/", router);

  mount.mount("/viewer", Static::new(Path::new("viewer")));

  println!("Open http://localhost:5001/viewer/");
  Iron::new(mount).http("0.0.0.0:5001").unwrap();
}
