use std::{collections::HashMap, fs::{self, read_to_string, File}, io::{BufReader, Read, Write}, net::{TcpListener, TcpStream}, os::unix::fs::FileExt, u8, usize};

use crate::http::http::{HttpMethod, HttpStatusCode};

use super::{http::{HttpRequest, HttpResponse}, utils::{get_file_content, handle_request}};


type RouteFn = fn(HttpRequest, &mut HttpResponse);
type Routes = HashMap<String, RouteFn>;
pub struct Server {
  pub port:String,
  public:Option<String>,
  get_routes:Routes,
  post_routes:Routes,
  // delete_routes:Routes,
  // update_routes:Routes,
  
}

impl Server {
  pub fn new(port:&str)->Self{
    return Server {
      port:port.to_string(),
      public:None,
      get_routes:HashMap::new(),
      post_routes:HashMap::new(),
      // delete_routes:HashMap::new(),
      // update_routes:HashMap::new(),
    }
  }
  pub fn set_static(&mut self, path:&str){
    self.public = Some(path.to_string());
  }
  pub fn get(&mut self, path:&str, fun:RouteFn) {
    self.get_routes.insert(path.to_string(), fun);
  }
  pub fn post(&mut self, path:&str, fun:RouteFn) {
    self.post_routes.insert(path.to_string(), fun);
  }
  pub fn run(&self) {
    let addr = format!("127.0.0.1:{}", &self.port);
    let listener = TcpListener::bind(addr).unwrap();
    for stream in listener.incoming() {
      match stream {
        Ok(mut stream) => {
          let http_request = handle_request(&mut stream);
          match http_request {
            Ok(http_request)=>{
              match http_request.method {
                HttpMethod::GET=>{
                  let path = http_request.path.clone();
                  let mut http_response = HttpResponse::new();
                  if let Some(public) = &self.public {
                    let path = &path[1..];
                    if path.starts_with(public) {
                      self.serve_static_files(path, http_request, stream, http_response);
                      continue;
                    }
                  }
                  if let Some(fun) = self.get_routes.get(&path){
                    fun(http_request, &mut http_response);
                    let status_line = "HTTP/1.1 200 Ok";
                    let content = match http_response.body {
                      Some(body)=>body,
                      None=>"".to_string(),
                    };
                    let length = content.len();
                    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");
                    stream.write_all(response.as_bytes()).unwrap();
                    continue; 
                  } else {
                    let status_line = "HTTP/1.1 404 Not Found";
                    let content = format!("Route Not Found!");
                    let lenght = content.len();
                    let response = format!("{status_line}\r\nContent-Length: {lenght}\r\n\r\n{content}");
                    stream.write_all(response.as_bytes()).unwrap();
                    continue;
                  }
                }, 
                _=>{
                  let status_line = "HTTP/1.1 404 Not Found";
                  let content = format!("Method Not Found!");
                  let lenght = content.len();
                  let response = format!("{status_line}\r\nContent-Length: {lenght}\r\n\r\n{content}");
                  stream.write_all(response.as_bytes()).unwrap();
                  continue;
                }
              }

            },
            Err(err)=>{
              let status_line = "HTTP/1.1 500 Internal Server Error";
              let content = format!("{err}");
              let lenght = content.len();
              let response = format!("{status_line}\r\nContent-Length: {lenght}\r\n\r\n{content}");
              stream.write_all(response.as_bytes()).unwrap();
            }
          }
        }
        Err(err)=> {println!("Error in Stream: {}", err);}
      };
      println!("connection established");
    }
  }
  pub fn serve_static_files(&self, path:&str, request:HttpRequest, mut stream:TcpStream, mut response:HttpResponse){
    let file = File::open(path);
    response.setHeader("Content-Type".to_string(), get_file_content(path).to_string());
    match file {
      Ok(mut file)=>{
        let mut read_whole_file = true;
        let mut start_byte:u64 = 0;
        let mut end_byte:u64 = 0;
        let mut file_size:u64 = 0;
        let range = request.header.get("Range");
        if let Some(range) = range {
          if let Some((first, second)) = range.split_once("=") {
            if first=="bytes"{
              file_size = match file.metadata() {
                Ok(metadata)=>metadata.len(),
                _=>0,
              };
              if let Some((start, end)) = second.split_once("-"){
                start_byte = start.parse().unwrap_or(0);
                end_byte = end.parse().unwrap_or(start_byte + (1024*1024*1));
              };
              read_whole_file = false;
            }
          };

        } 
        if read_whole_file {
          let mut data:Vec<u8> = vec![];
          match file.read_to_end(&mut data) {
            Ok(_)=>{
              let status_line = format!("HTTP/1.1 {}", response.status);
              let length = data.len();
              response.setHeader("Content-Length".to_string(), length.to_string());
              let mut header = String::from("");
              for (key, value) in response.header.iter() {
                let head_str = format!("\r\n{key}:{value}");
                header.push_str(&head_str);
              }
              let response_header = format!("{status_line}{header}\r\n\r\n");
              self.send_response(&mut stream, response_header.as_bytes());
              self.send_response(&mut stream, &data);

            },
            Err(_err)=>{
              let status_line = format!("HTTP/1.1 {}", HttpStatusCode::Forbidden);
              let content = format!("Unable to read file");
              let lenght = content.len();
              let response = format!("{status_line}\r\nContent-Length: {lenght}\r\n\r\n{content}");
              self.send_response(&mut stream, response.as_bytes());
            }
          } 
        } else {
          if file_size <= end_byte {
            end_byte = file_size-1;
          };
          let buffer_size = ((end_byte - start_byte) as usize ) + 1;
          let mut data:Vec<u8> = vec![0u8; buffer_size];
          match file.read_exact_at(&mut data, start_byte) {
            Ok(_)=>{
              response.status = HttpStatusCode::PartialContent;
              let range = format!("bytes {}-{}/{}", start_byte, end_byte, if file_size==0{"*".to_string()} else {file_size.to_string()});
              response.setHeader("Content-Range".to_string(), range);
              let status_line = format!("HTTP/1.1 {}", response.status);
              let length = data.len();
              response.setHeader("Content-Length".to_string(), length.to_string());
              let mut header = String::from("");
              for (key, value) in response.header.iter() {
                let head_str = format!("\r\n{key}:{value}");
                header.push_str(&head_str);
              }
              let response_header = format!("{status_line}{header}\r\n\r\n");
              self.send_response(&mut stream, response_header.as_bytes());
              self.send_response(&mut stream, &data);
            }
            Err(_err)=>{
              println!("Error in reading file {}", _err);
              let status_line = format!("HTTP/1.1 {}", HttpStatusCode::Forbidden);
              let content = format!("Unable to read file");
              let lenght = content.len();
              let response = format!("{status_line}\r\nContent-Length: {lenght}\r\n\r\n{content}");
              self.send_response(&mut stream, response.as_bytes());
            }
          }
        } 
      },
      Err(_err) => {
        println!("Error in reading file {}", _err);
        let status_line = format!("HTTP/1.1 {}", HttpStatusCode::NotFound);
        let content = format!("File Not Found!");
        let lenght = content.len();
        let response = format!("{status_line}\r\nContent-Length: {lenght}\r\n\r\n{content}");
        self.send_response(&mut stream, response.as_bytes());
      }
    }
  }
  pub fn send_response(&self, stream:&mut TcpStream, data:&[u8]) {
    if let Err(err) = stream.write_all(data) {
      println!("Error on sending data: {}", err);
      return;
    };
    if let Err(err) = stream.flush() {
      println!("Error on flushing data: {}", err);
    }
  }
}

