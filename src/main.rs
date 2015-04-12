extern crate time;
extern crate rustc_serialize;

mod runner;

fn main() {
  let r = runner::Runner;

  let mut args = std::env::args();
  let _ = args.next().unwrap();
  let cmd = args.next().unwrap();

  r.run(cmd.as_ref(), args.collect());
}
