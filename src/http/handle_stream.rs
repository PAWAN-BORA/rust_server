use std::{fs::File, io::{Read, Write}, net::TcpStream, os::unix::fs::FileExt, sync::{Arc, RwLock}, u64, u8, usize};

use super::{http::{HttpMethod, HttpRequest, HttpResponse, HttpStatusCode}, server::{RouteFn, Server}, utils::{get_file_content, handle_request}};



pub(crate) struct HandleStream {
  stream:TcpStream,
  server:Arc<RwLock<Server>>,
}

impl HandleStream {
  pub(crate) fn new(stream:TcpStream, server:Arc<RwLock<Server>>)->Self {
    HandleStream {
      stream,
      server
    }
  }  
  pub(crate) fn parse(&mut self){
    match handle_request(&mut self.stream) {
      Ok(http_request)=>{
        match http_request.method {
          HttpMethod::OPTIONS=> {
            self.handle_option_request();
          }
          HttpMethod::GET => {
            self.handle_get_request(http_request);
          }
          HttpMethod::POST=> {
            self.handle_post_request(http_request);
          }
          HttpMethod::DELETE=> {
            self.not_found("Delete method not found!");
          }
          HttpMethod::PUT=> {
            self.not_found("PUt method not found");
          }
          HttpMethod::HEAD=> {
            let mut response = HttpResponse::new();
            response.status = HttpStatusCode::NoContent;
            let header = self.get_header(&response);
            self.send_response(header.as_bytes());
          }
          HttpMethod::PATCH=> {
            self.not_found("Patch method not found!");
          }
          HttpMethod::OTHER(val) =>{
            let method_str = format!("{} method not found!", val);
            self.not_found(&method_str);
          }
        } 
      }
      Err(err)=>{
        self.server_error(&err);
      }
    }
    
  }

  fn get_path_fun(&self, route_name:&str, request:&HttpRequest)->Result<(String, Option<RouteFn>), String> {
    let server = match self.server.read() {
      Ok(server)=>server,
      Err(_err)=>{
        return Err("Unable to read server".to_string());
      }
    };
    if route_name=="get" {
      if let Some(public) = &server.public {
        let path = &request.path[1..];
        if path.starts_with(public) {
         return Ok((path.to_string(), None));
        }
      }
      if let Some(&fun) = &server.get_routes.get(&request.path){
        return Ok((request.path.clone(), Some(fun)));
      };
    } else if route_name=="post" {
      if let Some(&fun) = &server.post_routes.get(&request.path) {
        return Ok((request.path.clone(), Some(fun)));
      }
    }

    return Ok(("".to_string(), None));
  }

  fn handle_get_request(&mut self, request:HttpRequest){
    let (route, fun) = match self.get_path_fun("get", &request) {
      Ok((route, fun))=>(route, fun),
      Err(err)=>{
        self.server_error(&err);
        return;
      }
    };
    if let Some(fun) = fun {
      let mut response = HttpResponse::new();
      fun(request, &mut response);
      let headers = self.get_header(&response);
      let content = match &response.body {
        Some(body)=>body.clone(),
        None=>"".to_string()
      };
      response.set_header("Content-Length".to_string(), content.len().to_string());
      self.send_response(headers.as_bytes());
      self.send_response(content.as_bytes());
      // return;
    } else if route.is_empty() {
      self.not_found("File Not Found!");
    } else {
      self.serve_static_files(&route, request);

    }
  } 
  fn handle_post_request(&mut self, request:HttpRequest){
    let (route, fun) = match self.get_path_fun("post", &request) {
      Ok((route, fun))=>(route, fun),
      Err(err)=>{
        self.server_error(&err);
        return;
      }
    };
    if let Some(fun) = fun {
      let mut response = HttpResponse::new();
      fun(request, &mut response);
      let content = match &response.body {
        Some(body)=>body.clone(),
        None=>"".to_string()
      };
      response.set_header("Content-Length".to_string(), content.len().to_string());
      let headers = self.get_header(&response);
      self.send_response(headers.as_bytes());
      self.send_response(content.as_bytes());
    } else if route.is_empty() {
      self.not_found("Route Not Found!");
    } else {
      self.server_error("Error on finding routes!");
    }
  }
  fn handle_option_request(&mut self){
    let mut response = HttpResponse::new();
    response.status = HttpStatusCode::NoContent;
    let headers = self.get_header(&response);
    self.send_response(headers.as_bytes());
  }
  fn get_range(&self, request:&HttpRequest)->Option<(u64, u64)>{
    let chunk_size = 1024*1024*1;
    if let Some(range) = request.header.get("Range"){
      if let Some((first, second)) = range.split_once("=") {
        if first=="bytes"{
          if let Some((start, end)) = second.split_once("-"){
            let start_byte = start.parse().unwrap_or(0);
            let end_byte = end.parse().unwrap_or(start_byte + chunk_size);
            return Some((start_byte, end_byte))
          };
        };
      };
    };
    None
  }
  fn serve_static_files(&mut self, path:&str, request:HttpRequest){
    let mut response = HttpResponse::new();
    let file = File::open(path);
    response.set_header("Content-Type".to_string(), get_file_content(path).to_string());
    match file {
      Ok(file)=>{
        if let Some((start_byte, end_byte)) = self.get_range(&request) {
          let file_size = match file.metadata() {
            Ok(metadata)=>metadata.len(),
            _=>0,
          };
          self.send_chunk(start_byte, end_byte, file_size, file, response);
          return;
        };

        self.send_whole_file(file, response);
        return;
      },
      Err(err)=>{
        println!("Error in reading file {}", err);
        self.server_error(&err.to_string());
      }
    }

  }
  fn send_whole_file(&mut self, mut file:File, mut response:HttpResponse){
    let mut data:Vec<u8> = vec![];
    match file.read_to_end(&mut data){
      Ok(_)=>{
        response.set_header("Content-Length".to_string(), data.len().to_string());
        let headers = self.get_header(&response);
        self.send_response(headers.as_bytes());
        self.send_response(&data);
      },
      Err(err)=>{
        println!("Error on reading file: {err}");
        self.server_error(&err.to_string());
      }
    }
  }
  fn send_chunk(&mut self, start_byte:u64, mut end_byte:u64, file_size:u64, file:File, mut response:HttpResponse) {
    if file_size <= end_byte {
      end_byte = file_size-1;
    };
    let buffer_size = (end_byte - start_byte) as usize + 1;
    let mut data:Vec<u8> = vec![0; buffer_size];
    match file.read_exact_at(&mut data, start_byte){
      Ok(_)=>{
        response.status = HttpStatusCode::PartialContent;
        let range = format!("bytes {}-{}/{}", start_byte, end_byte, if file_size==0{"*".to_string()}else{file_size.to_string()});
        response.set_header("Content-Range".to_string(), range);
        response.set_header("Content-Length".to_string(), data.len().to_string());
        let headers = self.get_header(&response);
        self.send_response(headers.as_bytes());
        self.send_response(&data);
      }
      Err(err)=>{
        println!("Error on reading chunk: {err}");
        self.server_error(&err.to_string());
      }
    };

  }
  fn server_error(&mut self, err:&str){
    let mut response = HttpResponse::new();
    response.status = HttpStatusCode::InternalServerError;
    response.set_header("Content-Length".to_string(), err.len().to_string());
    let header_part = self.get_header(&response);
    self.send_response(header_part.as_bytes());
    self.send_response(err.as_bytes());
  }
  fn not_found(&mut self, err:&str){
    let mut response = HttpResponse::new();
    response.status = HttpStatusCode::NotFound;
    response.set_header("Content-Length".to_string(), err.len().to_string());
    let header_part = self.get_header(&response);
    self.send_response(header_part.as_bytes());
    self.send_response(err.as_bytes());
  }
  fn get_header(&self, response:&HttpResponse)->String{
    let status_line = format!("HTTP/1.1 {}", response.status);
    let mut header = String::from("");
    for (key, value) in response.header.iter() {
      let head_str = format!("\r\n{key}:{value}");
      header.push_str(&head_str);
    }
    format!("{}{}\r\n\r\n", status_line, header)
  }
  fn send_response(&mut self, data:&[u8]){
    if let Err(err) = self.stream.write_all(data) {
      println!("Error on sending data: {}", err);
      return;
    };
    if let Err(err) = self.stream.flush() {
      println!("Error on flushing data: {}", err);
    }

  }
}
