use std::fs;
use std::io::Error;

pub fn find_files(search_path: &str) -> Result<Vec<String>, Error> {
  let mut files: Vec<String> = Vec::new();
  try!(find_files_(search_path, &mut files));
  Ok(files)
}

fn find_files_(search_path: &str, files: &mut Vec<String>) -> Result<(), Error> {
  for entry in try!(fs::read_dir(search_path)) {
    let en = try!(entry);
    let pen = en.path();
    let path = pen.as_path();

    if path.is_dir() {
      try!(find_files_(path.to_str().unwrap(), files));
    } else if path.is_file() {
      match path.to_str() {
        Some(s) => files.push(s.to_string()),
        None => println!("Ignoring non utf-8 named file: {:?}", path)
      }
    }
  }
  Ok(())
}
