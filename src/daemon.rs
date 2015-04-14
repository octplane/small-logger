use std::path::Path;

use staticfile::Static;

use iron::prelude::*;
use iron::status;
use mount::Mount;


use router::{Router};


fn handler(req: &mut Request) -> IronResult<Response> {
    let ref query = req.extensions.find::<Router>().unwrap().find("method").unwrap_or("/");
    println!("{}", query);
    Ok(Response::with(status::Ok))
}

pub fn startup() {
    let mut mount = Mount::new();

    mount.mount("/logs", Static::new(Path::new("logs")));

    let mut router = Router::new();
    router.get("/:method", handler);
    mount.mount("/api/1/", router);
    
    Iron::new(mount).http("0.0.0.0:5001").unwrap();
}


pub struct Daemon;
