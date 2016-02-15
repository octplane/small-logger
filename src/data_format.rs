#[allow(dead_code)]

extern crate time;

use std::collections::HashMap;
use rustc_serialize::{Encoder, Encodable, Decoder, Decodable};
use rustc_serialize::json::decode;
use std::fs::File;
use std::io::Error;
use std::io::BufReader;
use std::io::BufRead;
use std::convert::AsRef;

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

impl JsonTime {
  fn from_string(time_to_ms: String) -> Result<JsonTime, String> {
    let format = "%Y-%m-%d %T.%f";
    // Add ns to time
    let time = time_to_ms + "000000";
    println!("{}", time);
    match time::strptime(time.as_ref(), format) {
      Ok(ts) => Ok(JsonTime{time: ts}),
      Err(_) => Err(format!("Unable to parse {} as {}", time, format))
    }
  }

  fn to_string(&self) -> String {
    let format = "%Y-%m-%d %T.%f";
    let mut ts = time::strftime(format, &self.time).ok().unwrap();
    let l = ts.len();
    ts.truncate(l-6);
    ts
  }
}

impl Decodable for JsonTime {
  fn decode<D: Decoder>(d: &mut D) -> Result<JsonTime, D::Error> {
    let cropped_time = try!(d.read_str());
    let format = "%Y-%m-%d %T.%f";

    let time = cropped_time + "000000";
    match time::strptime(time.as_ref(), format) {
      Ok(ts) => Ok(JsonTime{time: ts}),
      Err(_) => Err(d.error(format!("Unable to parse {} as {}", time, format).as_ref()))
    }
  }
}

impl Encodable for JsonTime {
  fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    let format = "%Y-%m-%d %T.%f";
    let mut ts = time::strftime(format, &self.time).ok().unwrap();
    let l = ts.len();
    ts.truncate(l-6);
    s.emit_str(ts.as_ref())
  }
}

#[test]
fn encodeDecodeJsonTime() {
  let t = time::at(time::Timespec::new(236928791, 113000000));
  let jt = JsonTime{time: t};
  let witness = "1977-07-05 07:33:11.113";
  assert_eq!(jt.to_string(), witness);
  assert_eq!(JsonTime::from_string(witness.to_string()).unwrap().time, jt.time);
}


#[derive(Debug)]
#[derive(Clone)]
#[derive(RustcDecodable)]
#[derive(RustcEncodable)]
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
#[derive(RustcEncodable)]
pub struct DeserializableTimestampedLine {
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
  start_time: time::Tm,
  end_time: Option<time::Tm>,
}

impl FileMeta {
  pub fn fast_meta(source: String) -> Result<FileMeta, Error> {
    println!("{}", source);
    let f = try!(File::open(source.clone()));
    let mut t = BufReader::new(f);
    let mut l = String::new();
    match t.read_line(&mut l) {
      Ok(_) => {
        match decode::<DeserializableTimestampedLine>(l.as_ref()) {
          Ok(line) => println!("{:?}", line),
          Err(e) => panic!("Failed reading and decoding line {}: {}", l, e)
        }
      }
      Err(e) => panic!("{}", e)
    }

    Ok(FileMeta{source: LogSource::BuildSystem, start_time: time::now(), end_time: None})
  }
}
