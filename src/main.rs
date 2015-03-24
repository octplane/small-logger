#![feature(core)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate time;

use std::io::prelude::*;
use std::os::unix::prelude::*;

use std::io::BufReader;
use std::thread;
use std::process::{Command, Stdio};
use std::collections::HashMap;


use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum LogSource {
  ControlSystem,
  StdOut,
  StdErr,
  BuildSystem,
}

impl LogSource {
  pub fn to_string(&self) -> String {
    match *self {
      LogSource::ControlSystem => "Control System",
      LogSource::StdOut => "stdout",
      LogSource::StdErr => "stderr",
      LogSource::BuildSystem => "Log System",
    }.to_string()
  }
}

pub fn to_string_ts(ti: time::Tm) -> String {
    let format = "%Y-%m-%d %T.%f";
    let mut ts = time::strftime(format, &ti).ok().unwrap();
    let l = ts.len();
    ts.truncate(l-6);
    ts
}

#[derive(Debug)]
pub struct TimestampedLine {
  source: LogSource,
  time: time::Tm,
  content: String,
}

impl TimestampedLine {
  pub fn tsl(source: LogSource, time: time::Tm, content: String) -> HashMap<String, String> {
    let mut line = HashMap::new();
    line.insert("source".to_string(), source.to_string());
    line.insert("time".to_string(), to_string_ts(time.to_utc()));
    line.insert("content".to_string(), content);

    line
  }

  pub fn msg(reason: String) -> HashMap<String, String> {
    TimestampedLine::tsl(LogSource::BuildSystem, time::now(), reason)
  }
  pub fn stop_writer() -> HashMap<String, String> {
    TimestampedLine::tsl(LogSource::ControlSystem, time::now(), "stop".to_string())
  }
}

pub struct Writer;

impl Writer {
  fn run(&self, receiver: Receiver<HashMap<String, String>> ) {
    let mut stop = 3;
    while stop != 0 {
      match receiver.recv() {
        Ok(tl) => {
          // This is a stop command, decrement counter. We need to wait for main, stderr, stdout before stopping
          if tl.get("source") == Some(&LogSource::ControlSystem.to_string()) && tl.get("content") == Some(&"stop".to_string()) {
              stop = stop - 1;
          } else {
            println!("{:?}", tl);
          }
        }
        Err(e) => println!("Receive error: {}", e),
      }
    }
  }
}

pub struct Runner;

impl Runner {
	fn run(&self, cmd: &str, parms: Vec<String>) {

    let (sender, receiver) = channel();

    let writer = Writer;
    let th = thread::spawn(move || {
      writer.run(receiver);
    });


    sender.send(TimestampedLine::msg(format!("Processing {}", cmd))).unwrap();

    let start = time::now();
    let mut child = match Command::new(&cmd)
      .args(parms.as_slice())
      .current_dir(".")
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn() {
        Err(why) => panic!("couldn't spawn {}: {}", &cmd, why.to_string()),
        Ok(child) => child,
    };

    //https://github.com/rust-lang/rust/blob/b83b26bacb6371173cdec6bf68c7ffa69f858c84/src/libstd/process.rs
    fn read_timestamped_lines<T: Read + Send + 'static>(stream: Option<T>, source: LogSource, sender: Sender<HashMap<String, String>>) {
      match stream {
        Some(stream) => {
          thread::spawn(move || {
            let mut br = BufReader::with_capacity(64, stream);
            while {
              let mut line = String::new();
              let ok = match br.read_line(&mut line) {
                Ok(0) => false,
                Ok(_) => true,
                Err(e) => {println!("Something went wrong while reading the data: {}", e.to_string()); false}
              };
              if ok {
                let now = time::now();
                sender.send(TimestampedLine::tsl(source.clone(), now, line)).unwrap();
              }
              ok
            } {}
            sender.send(TimestampedLine::stop_writer()).unwrap();
          });
        }
        None => sender.send(TimestampedLine::msg("Stream is None, not reading anything.".to_string())).unwrap()
      }
    }
    read_timestamped_lines(child.stdout.take(), LogSource::StdOut, sender.clone());
    read_timestamped_lines(child.stderr.take(), LogSource::StdErr, sender.clone());
    let status = child.wait();
    let end = time::now();


    sender.send(
      match status {
        Ok(es) => if es.success() {
          TimestampedLine::msg("Process completed successfully.".to_string())
        } else if let Some(ecode) = es.code() {
          TimestampedLine::msg(format!("Process completed with error code {}.", ecode))
        } else if let Some(esignal) = es.signal() {
          TimestampedLine::msg(format!("Process aborted with signal {}.", esignal))
        } else {
          TimestampedLine::msg("Non-reachable condition reached. Something's wrong".to_string())
        },
        Err(run_error) => TimestampedLine::msg(format!("Something went wrong while getting the command output: {:?}", run_error)),
      }
    ).unwrap();

    sender.send(TimestampedLine::stop_writer()).unwrap();
    let _ = th.join();
	}
}

#[derive(Debug)]
pub struct Invocation {
  command: String,
  parameters: Vec<String>,
  started: time::Tm,
  ended: time::Tm,
  exitstatus: Option<isize>,
  exitsignal: Option<isize>,
  logs: Vec<TimestampedLine>
}

fn main() {
  let r = Runner;

  r.run("ls", Vec::<String>::new());
}
