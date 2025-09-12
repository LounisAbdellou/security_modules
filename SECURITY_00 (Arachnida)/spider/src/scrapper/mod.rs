use std::{
  collections::{HashSet, VecDeque},
  path::PathBuf,
};

use regex::Regex;

use crate::scrapper::file_manager::FileManager;

mod file_manager;

pub struct Scrapper {
  client: reqwest::blocking::Client,
  file_manager: FileManager,
  recursion_max: u64,
  is_recursive: bool,
}

impl Scrapper {
  pub fn new() -> Self {
    let mut destination_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    destination_path.push("data");

    Self {
      client: reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::default())
        .timeout(std::time::Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build client"),
      file_manager: FileManager::from(destination_path),
      recursion_max: 1,
      is_recursive: false,
    }
  }

  fn handle_recursion_depth(&mut self, depth: Option<String>) {
    let depth = match depth {
      Some(value) => value.parse::<u64>(),
      None => {
        println!("Error: Wrong argument format: no detph given");
        std::process::exit(1)
      },
    };

    if depth.is_ok() {
      self.recursion_max = depth.unwrap();
    } else {
      println!("Error: Wrong argument format: depth wasn't a number");
      std::process::exit(1)
    }
  }

  fn extract_pattern(&self, text: &String, pattern: &'static str) -> String {
    let regex = Regex::new(pattern).expect("Failed to load domain regex");

    match regex
      .captures(text.as_str())
      .and_then(|capture| capture.get(1))
      .map(|res| res.as_str())
    {
      Some(name) => name.to_string(),
      None => text.clone(),
    }
  }

  fn find_pattern(&self, text: &String, pattern: &'static str) -> String {
    let regex = Regex::new(pattern).expect("Failed to load domain regex");

    match regex.find(text.as_str()).map(|res| res.as_str()) {
      Some(name) => name.to_string(),
      None => text.clone(),
    }
  }

  fn parse_img(&self, url: &String, data: &String, dirname: &String) {
    let regex =
      Regex::new(r#"<img[^>]+src=["']([^"']+\.(jpg|jpeg|png|gif|bmp))["']"#)
        .expect("Failed to load img regex");

    let img_sources = regex
      .captures_iter(data)
      .map(|capture| capture[1].to_string())
      .collect::<HashSet<String>>();

    for mut source in img_sources {
      if let None = source.find("://") {
        let first_char = source.chars().next().expect("Failed to get char");

        if first_char != '/' {
          source.insert(0, '/');
        }

        source.insert_str(0, url);
      }

      let img_data = match self.fetch_img(&source) {
        Ok(data) => data,
        Err(err) => {
          println!("{err}");
          continue;
        },
      };

      let filename_regex =
        Regex::new(r"[^\/]+$").expect("Failed to load filename regex");

      let filename =
        match filename_regex.find(source.as_str()).map(|res| res.as_str()) {
          Some(name) => name.to_string(),
          None => source.clone(),
        };

      self.file_manager.create_file(dirname, &filename, img_data);
    }
  }

  fn fetch_img(&self, url: &String) -> Result<Vec<u8>, String> {
    let response = match self.client.get(url).send() {
      Ok(res) => res,
      Err(err) => return Err(format!("reqwest: {err}")),
    };

    let data = match response.bytes() {
      Ok(d) => d.to_vec(),
      Err(err) => return Err(format!("reqwest: {err}")),
    };

    Ok(data)
  }

  fn fetch_html(&self, url: &String, recursion_level: u64) {
    if recursion_level > self.recursion_max {
      return;
    }

    let response = match self.client.get(url).send() {
      Ok(res) => res,
      Err(err) => return println!("reqwest: {err}"),
    };

    let data = match response.text() {
      Ok(d) => d,
      Err(err) => return println!("reqwest: {err}"),
    };

    let domain_regex = Regex::new(r"^(?:https?:\/\/)?([^\/]+)").unwrap();
    let dirname = match domain_regex
      .captures(url.as_str())
      .and_then(|capture| capture.get(1))
      .map(|res| res.as_str())
    {
      Some(name) => name.to_string(),
      None => url.clone(),
    };

    self.file_manager.create_sub_dir(&dirname);
    self.parse_img(url, &data, &dirname);
  }

  pub fn run(&mut self, mut arguments: VecDeque<String>) {
    while arguments.len() > 1 {
      let current_arg = arguments.pop_front().expect("Failed to get argurment");

      match current_arg.as_str() {
        "-r" => self.is_recursive = true,
        "-l" => self.handle_recursion_depth(arguments.pop_front()),
        _ => {
          println!("Error: Wrong argument format: {current_arg}");
          std::process::exit(1)
        },
      };
    }

    self.file_manager.create_work_dir();

    match arguments.pop_front() {
      Some(url) => self.fetch_html(&url, 1),
      None => {
        println!("Error: Wrong argument format: no url given");
        std::process::exit(1)
      },
    }
  }
}
