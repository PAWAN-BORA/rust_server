use std::{collections::HashMap, fs::{self, read_to_string, File}, io::{BufReader, Read, Write}, net::{TcpListener, TcpStream}};

use crate::http::http::HttpMethod;

use super::{http::{HttpRequest, HttpResponse}, utils::handle_request};


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
                      // self.send_response(stream, http_request, http_response);
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
    let file = read_to_string(path);
    match file {
      Ok(content)=>{
        let status_line = format!("HTTP/1.1 {}", response.status);
        let length = content.len();
        response.setHeader("Content-Length".to_string(), length.to_string());
        let mut header = String::from("");
        for (key, value) in response.header.iter() {
          let head_str = format!("\r\n{key}:{value}");
          header.push_str(&head_str);
        }
        let response = format!("{status_line}{header}\r\n\r\n{content}");
        stream.write_all(response.as_bytes()).unwrap();
      },
      _ => {
        let status_line = "HTTP/1.1 404 Not Found";
        let content = format!("File Not Found!");
        let lenght = content.len();
        let response = format!("{status_line}\r\nContent-Length: {lenght}\r\n\r\n{content}");
        stream.write_all(response.as_bytes()).unwrap();
      }
    }
  }
  pub fn send_response(&self, mut stream:TcpStream, request:HttpRequest, response:HttpResponse) {
    let status = response.status;
    let status_line = format!("HTTP/1.1 {status}");
    // let method = request.method;
    // let path = request.path;
    // let version = request.version;
    let content = read_to_string("public/index.html").unwrap();//format!("method:{method}, path:{path}, version:{version}");
    let lenght = content.len();
    let response = format!("{status_line}\r\nContent-Length: {lenght}\r\n\r\n{content}");
    stream.write_all(response.as_bytes()).unwrap();

  }

}

