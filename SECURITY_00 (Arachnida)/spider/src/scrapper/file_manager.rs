use std::{fs::File, io::Write, path::PathBuf};

pub struct FileManager {
  work_dir_path: PathBuf,
}

impl FileManager {
  pub fn from(path: PathBuf) -> Self {
    Self {
      work_dir_path: path,
    }
  }

  pub fn create_file(
    &self,
    dirname: &String,
    filename: &String,
    data: Vec<u8>,
  ) {
    let mut file_path = self.work_dir_path.clone();
    file_path.push(dirname);
    file_path.push(filename);

    let mut file = match File::create(file_path) {
      Ok(file) => file,
      Err(err) => return println!("fs: {err}"),
    };

    if let Err(err) = file.write_all(&data) {
      return println!("fs: {err}");
    }
  }

  pub fn create_dir(&self, dir_path: &PathBuf) {
    if dir_path.exists() && dir_path.is_dir() {
      return;
    }

    let dir = std::fs::create_dir(dir_path);

    if let Err(err) = dir {
      println!("fs: {err}");
      std::process::exit(1)
    }
  }

  pub fn create_work_dir(&self) {
    self.create_dir(&self.work_dir_path);
  }

  pub fn create_sub_dir(&self, dirname: &String) {
    let mut dir_path = self.work_dir_path.clone();
    dir_path.push(dirname);

    self.create_dir(&dir_path);
  }
}
