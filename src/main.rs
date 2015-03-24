#![feature(core)]
#![feature(std_misc)]

extern crate time;
extern crate "rustc-serialize" as rustc_serialize;

use std::io::prelude::*;
use std::os::unix::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::fs::create_dir_all;
use std::path::Path;
use std::thread;
use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};

use rustc_serialize::json;

#[derive(Debug)]
#[derive(Clone)]
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
  fn run(&self, destination_file:String,  receiver: Receiver<HashMap<String, String>> ) {
    let destination = Path::new(destination_file.as_slice());
    std::fs::create_dir_all(destination.parent().unwrap()).ok();
    println!("Writing to {}.", destination_file);
    match File::create(destination) {
      Ok(mut file) => {
        let mut stop = 3;
        while stop != 0 {
          match receiver.recv() {
            Ok(tl) => {
              // This is a stop command, decrement counter. We need to wait for main, stderr, stdout before stopping
              if tl.get("source") == Some(&LogSource::ControlSystem.to_string()) && tl.get("content") == Some(&"stop".to_string()) {
                  stop = stop - 1;
              } else {
                let encoded = json::encode(&tl).unwrap();
                println!("{:?}", tl);
                file.write_all(&encoded.into_bytes()).ok();
                file.write_all(b"\n").ok();
              }
            }
            Err(e) => println!("Receive error: {}", e),
          }
        }
      },
      Err(e) => panic!("Unable to open output file {}: {}", destination_file, e.to_string())
    }
  }
}

pub struct Runner;

impl Runner {
	fn run(&self, cmd: &str, parms: Vec<String>)  {
    let start = time::now();

    let cmd_path = Path::new(cmd);
    let fname = match cmd_path.file_name() {
      Some(f) => {
        let mut pth = time::strftime("./logs/%Y/%m/%d/", &start).ok().unwrap();
        let postfix = time::strftime("-%T.ajson", &start).ok().unwrap();
        let osname = f.to_string_lossy().into_owned();
        pth.push_str(&osname);
        pth.push_str(&postfix);
        pth
      },
      None => panic!("Unable to find file name for {}", cmd),
    };

    let (sender, receiver) = channel();
    let writer = Writer;
    let th = thread::spawn(move || {
      writer.run(fname, receiver);
    });

    sender.send(TimestampedLine::msg(format!("Processing {}", cmd))).unwrap();


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
                sender.send(TimestampedLine::tsl(source.clone(), now, line[0..line.len()-1].to_string())).unwrap();
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

    let duration = (end-start).num_milliseconds().to_string();

    let mut end_line = match status {
    Ok(es) => if es.success() {
        let mut msg = TimestampedLine::msg("Process completed successfully.".to_string());
        msg.insert("completed".to_string(), "0".to_string());
        msg
      } else if let Some(ecode) = es.code() {
        TimestampedLine::msg(format!("Process completed with error code {}.", ecode))
      } else if let Some(esignal) = es.signal() {
        TimestampedLine::msg(format!("Process aborted with signal {}.", esignal))
      } else {
        TimestampedLine::msg("Non-reachable condition reached. Something's wrong".to_string())
      },
      Err(run_error) => TimestampedLine::msg(format!("Something went wrong while getting the command output: {:?}", run_error)),
    };

    end_line.insert("duration".to_string(), duration);
    sender.send(end_line).unwrap();

    sender.send(TimestampedLine::stop_writer()).unwrap();
    // Wait for writer thread to complete its writes
    let _ = th.join();
	}
}

fn main() {
  let r = Runner;

  r.run("/bin/ls", Vec::<String>::new());
}
