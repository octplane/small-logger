#![allow(dead_code)]

extern crate rustc_serialize;
extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate router;
extern crate getopts;

mod data_format;
mod runner;
mod daemon;
mod api;

use std::env;
use getopts::Options;

extern crate time;


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [-d] [-s] [COMMAND WITH PARAMETERS]", program);
    print!("{}", opts.usage(&brief));
}


fn main() {

  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();

  let mut opts = Options::new();
  opts.optflag("s", "server", "start web server");
  opts.optflag("d", "daemonize", "Daemonize the webserver, implies -s");
  opts.optflag("h", "help", "print this help menu");

  let matches = match opts.parse(&args[1..]) {
    Ok(m) => { m }
    Err(f) => { panic!(f.to_string()) }
  };
  if matches.opt_present("h") {
    print_usage(&program, opts);
    return;
  }
  if matches.opt_present("d") || matches.opt_present("s") {
    daemon::startup();
  } else {
    if matches.free.len() > 0 {
      let r = runner::Runner;

      let mut cmd = matches.free.clone();
      let params = cmd.split_off(1);

      r.run(cmd[0].as_str(), params);
    } else {
      print_usage(&program, opts);
      return;
    }
  };



}
