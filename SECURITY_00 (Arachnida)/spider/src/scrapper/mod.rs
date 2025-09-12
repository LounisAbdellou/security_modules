use std::{
  collections::{HashSet, VecDeque},
  path::PathBuf,
};

use quick_xml::escape::unescape;
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

  fn parse_imgs(&self, url: &String, data: &String, dirname: &String) {
    let imgs_regex =
      Regex::new(r#"<img[^>]+src=["']([^"']+\.(jpg|jpeg|png|gif|bmp))["']"#)
        .expect("Failed to load img regex");

    let sources = imgs_regex
      .captures_iter(data)
      .map(|capture| capture[1].to_string())
      .collect::<HashSet<String>>();

    for source in sources {
      let mut unescaped_source = match unescape(source.as_str()) {
        Ok(s) => s.to_string(),
        Err(err) => {
          println!("quick-xml: {err}");
          continue;
        },
      };

      if let None = unescaped_source.find("://") {
        let first_char =
          unescaped_source.chars().next().expect("Failed to get char");

        if first_char != '/' {
          unescaped_source.insert(0, '/');
        }

        unescaped_source.insert_str(0, url);
      }

      let filename_regex =
        Regex::new(r#"[^\/]+$"#).expect("Failed to load filename regex");

      let filename = match filename_regex
        .find(unescaped_source.as_str())
        .map(|res| res.as_str())
      {
        Some(name) => name.to_string(),
        None => unescaped_source.clone(),
      };

      if !self.file_manager.file_exist(dirname, &filename) {
        let img_data = match self.fetch_img(&unescaped_source) {
          Ok(data) => data,
          Err(err) => {
            println!("{err}");
            continue;
          },
        };

        self.file_manager.create_file(dirname, &filename, img_data);
      }
    }
  }

  fn parse_links(&self, url: &String, data: &String, recursion_level: u64) {
    let links_regex =
      Regex::new(r#"<a\s+(?:[^>]*?\s+)?href=([\"']?)([^"\'\s>]+)"#)
        .expect("Failed to load img regex");

    let links = links_regex
      .captures_iter(data)
      .map(|capture| capture[2].to_string())
      .collect::<HashSet<String>>();

    for link in links {
      let mut unescaped_link = match unescape(link.as_str()) {
        Ok(l) => l.to_string(),
        Err(err) => {
          println!("quick-xml: {err}");
          continue;
        },
      };

      if let None = unescaped_link.find("://") {
        unescaped_link.insert_str(0, url);
      }

      if recursion_level < self.recursion_max {
        self.fetch_html(&unescaped_link, recursion_level + 1);
      }
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
    let response = match self.client.get(url).send() {
      Ok(res) => res,
      Err(err) => return println!("reqwest: {err}"),
    };

    let data = match response.text() {
      Ok(d) => d,
      Err(err) => return println!("reqwest: {err}"),
    };

    let domain_regex = Regex::new(r#"^(?:https?:\/\/)?([^\/]+)"#).unwrap();
    let dirname = match domain_regex
      .captures(url.as_str())
      .and_then(|capture| capture.get(1))
      .map(|res| res.as_str())
    {
      Some(name) => name.to_string(),
      None => url.clone(),
    };

    self.file_manager.create_sub_dir(&dirname);
    self.parse_imgs(url, &data, &dirname);
    self.parse_links(url, &data, recursion_level);
  }

  pub fn run(&mut self, mut arguments: VecDeque<String>) {
    while arguments.len() > 1 {
      let current_arg = arguments.pop_front().expect("Failed to get argurment");

      match current_arg.as_str() {
        "-l" => self.handle_recursion_depth(arguments.pop_front()),
        "-r" => {
          self.is_recursive = true;
          self.recursion_max = 5;
        },
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
