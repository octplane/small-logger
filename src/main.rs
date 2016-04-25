#![allow(dead_code)]

extern crate rustc_serialize;
extern crate getopts;

mod data_format;
mod runner;

use std::env;
use getopts::Options;

extern crate time;


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [-d log_folder] [-n process_name] [COMMAND WITH PARAMETERS]", program);
    print!("{}", opts.usage(&brief));
}


fn main() {

  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();

  let mut opts = Options::new();
  opts.optopt("d", "directory", "root log folder. Default is ./logs", "DIRECTORY");
  opts.optopt("n", "name", "pretty name for the process. Should be time invariant and filesystem friendly. Default is process", "PROCESS");
  opts.optopt("c", "change_directory", "PWD for the process. Default is .", "WORKING_DIRECTORY");
  opts.optflag("h", "help", "print this help menu.");

  let matches = match opts.parse(&args[1..]) {
    Ok(m) => { m }
    Err(f) => { panic!(f.to_string()) }
  };
  if matches.opt_present("h") {
    print_usage(&program, opts);
    return;
  }

  let directory = matches.opt_str("d").unwrap_or(String::from("./logs"));
  let process = matches.opt_str("n").unwrap_or(String::from("process"));
  let working_directory = Some(matches.opt_str("c").unwrap_or(String::from(".")));

  // if matches.opt_present("d") || matches.opt_present("s") {
  if matches.free.len() > 0 {
    let r = runner::Runner;

    let mut cmd = matches.free.clone();
    let params = cmd.split_off(1);

    let _dummy = r.run(cmd[0].as_str(), params, directory, process, working_directory);
  } else {
    print_usage(&program, opts);
    return;
  }

}
