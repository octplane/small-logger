extern crate time;

use std::collections::HashMap;

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
