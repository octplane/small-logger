#![feature(path_ext)]
#![feature(collections)]

extern crate time;
extern crate rustc_serialize;
extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate router;

mod data_format;
mod runner;
mod daemon;
mod api;

fn main() {
  daemon::startup();

  let r = runner::Runner;

  let mut args = std::env::args();
  let _ = args.next().unwrap();
  let cmd = args.next().unwrap();

  r.run(cmd.as_ref(), args.collect());
}
