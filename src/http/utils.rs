use std::{collections::HashMap, io::{BufRead, BufReader, Read, Write}, net::TcpStream, usize};
use super::http::{HttpMethod, HttpRequest};

pub(crate) fn get_status_data(status_line:String)->Result<(String, String, String), String>{
  let mut status_split = status_line.split(" ");
  let method = match status_split.next() {
    Some(method)=>method.to_string(),
    None => {
      return Err("Method Not Found!".to_string());
    }
  };
  let path = match status_split.next() {
    Some(path)=>path.to_string(),
    None => {
      return Err("Http Version Not Found!".to_string());
    }
  };
  let version = match status_split.next() {
    Some(version)=>version.to_string(),
    None => {
      return Err("Http Version Not Found!".to_string());
    }
  };
  return Ok((method, path, version));
}
pub(crate) fn handle_request(stream:&mut TcpStream)->Result<HttpRequest, String>{
  let mut buf_reader = BufReader::new(stream);
  let mut first_line = String::new();
  let line_res = buf_reader.read_line(&mut first_line);
  if let Ok(_status) = line_res {
    let (method, path, version) = get_status_data(first_line)?; 
    let mut header:HashMap<String, String> = HashMap::new();
    loop {
      let mut line = String::new();
      buf_reader.read_line(&mut line).unwrap();
      if line.trim().is_empty() {
        break;
      }
      let mut split_array = line.split(":");
      let key = match split_array.next() {
        Some(key)=>key.to_string(),
        _ => "".to_string(),
      };
      let value = match split_array.next() {
        Some(value)=>value.to_string(),
        _ => "".to_string(),
      };
      header.insert(key, value);
    }
    let content_lenght:usize = match header.get("Content-Length") {
      Some(value)=>{
        let val = value.trim().parse();
        match val {
          Ok(val)=>val,
          _=>0,
          
        }
      },
      None => 0,
    };
    let mut body:Option<String>= None;
    if content_lenght>0 {
      let mut body_vec = vec![0; content_lenght];
      match buf_reader.read_exact(&mut body_vec) {
        Ok(size)=>size,
        Err(err)=>{
          println!("Error in reading body: {err}");
          return Err("Error in reading body: {err}".to_string());
        }
      };
      body = Some(String::from_utf8(body_vec).unwrap());
    }

    let request = HttpRequest {
      header:header,
      method:HttpMethod::from_str(&method),
      version:version,
      path:path,
      body:body,
    };
    return Ok(request);
  } else {
    println!("http status line not found");
    return Err("http status line not found".to_string());
  }
}
