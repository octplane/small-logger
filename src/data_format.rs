extern crate time;

use std::collections::HashMap;
use rustc_serialize::{Decoder, Decodable};
use std::fs::File;
use std::io::Error;
use std::io::BufReader;
use std::io::BufRead;

trait Serializable {
    fn serialized(&self) -> String;
}

impl Serializable for time::Tm {
    fn serialized(&self) -> String {
        let format = "%Y-%m-%d %T.%f";
        let mut ts = time::strftime(format, &self).ok().unwrap();
        let l = ts.len();
        ts.truncate(l-6);
        ts
    }
}

#[derive(Debug)]
struct JsonTime {
    time: time::Tm
}


impl Decodable for JsonTime {
    fn decode<D: Decoder>(d: &mut D) -> Result<JsonTime, D::Error> {
        let v = JsonTime{time: time::now()};
        Ok(v)
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(RustcDecodable)]
pub enum LogSource {
  ControlSystem, // Used to control the writer process. Should not appear in the logs
  StdOut,
  StdErr,
  BuildSystem, // Used for system logs in the log files
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

#[derive(Debug)]
pub struct TimestampedLine {
  source: LogSource,
  time: time::Tm,
  content: String,
}

#[derive(Debug)]
#[derive(RustcDecodable)]
pub struct TimestampedLineD {
  source: LogSource,
  time: JsonTime,
  content: String,
}


impl TimestampedLine {
  pub fn tsl(source: LogSource, time: time::Tm, content: String) -> HashMap<String, String> {
    let mut line = HashMap::new();
    line.insert("source".to_string(), source.to_string());
    line.insert("time".to_string(), time.to_utc().serialized());
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

#[derive(Debug)]
pub struct FileMeta {
    source: LogSource,
    startTime: time::Tm,
    endTime: Option<time::Tm>,
}

impl FileMeta {
    pub fn fast_meta(source: String) -> Result<FileMeta, Error> {
        println!("{}", source);
        let f = try!(File::open(source.clone()));
        let mut t = BufReader::new(f);
        let mut l = String::new();
        match t.read_line(&mut l) {
            Ok(sz) => println!("Read first line of {} : {}", source,  l),
            Err(e) => panic!("{}", e)
        }

        Ok(FileMeta{source: LogSource::BuildSystem, startTime: time::now(), endTime: None})
    }
}
