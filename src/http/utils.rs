use std::{collections::HashMap, io::{BufRead, BufReader, Read, Write}, net::TcpStream, path::Path, usize};
use urlencoding::decode;

use super::http::{HttpMethod, HttpRequest};

fn get_status_data(status_line:String)->Result<(String, String, String), String>{
  let mut status_split = status_line.split(" ");
  let method = match status_split.next() {
    Some(method)=>method.to_string(),
    None => {
      return Err("Method Not Found!".to_string());
    }
  };
  let url = match status_split.next() {
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
  return Ok((method, url, version));
}
fn get_path_param(url:&String)->(String, HashMap<String, String>) {
  let mut params = HashMap::new();
  if let Some((path, param_str)) = url.split_once("?") {
    let param_str = match decode(param_str){
      Ok(val)=>{val.into()}
      Err(_)=>{String::from("")}
    };
    if !param_str.is_empty() {
      for param in param_str.split("&") {
        if let Some((key, value)) = param.split_once("="){
          params.insert(key.trim().to_string(), value.trim().to_string());
        };
      }
    }
    return (path.to_string(), params);
  } else {
    return (url.to_string(), params);
  };
}

pub(crate) fn handle_request(stream:&mut TcpStream)->Result<HttpRequest, String>{
  let mut buf_reader = BufReader::new(stream);
  let mut first_line = String::new();
  let line_res = buf_reader.read_line(&mut first_line);
  if let Ok(_status) = line_res {
    let (method, url, version) = get_status_data(first_line)?; 
    let (path, params) = get_path_param(&url);
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
      header.insert(key.trim().to_string(), value.trim().to_string());
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
          return Err(format!("Error in reading body: {err}"));
        }
      };
      body = Some(String::from_utf8(body_vec).unwrap());
    }

    let request = HttpRequest {
      header:header,
      params:params,
      method:HttpMethod::from_str(&method),
      version:version,
      path:path,
      body:body,
    };
    return Ok(request);
  } else {
    return Err("http status line not found".to_string());
  }
}

pub(crate) fn get_file_content(file_path:&str)->&'static str {
  let mime_types: HashMap<&str, &str> = HashMap::from([
        ("html", "text/html"),
        ("css", "text/css"),
        ("js", "application/javascript"),
        ("json", "application/json"),
        ("mp4", "video/mp4"),
        ("png", "image/png"),
        ("jpg", "image/jpeg"),
        ("jpeg", "image/jpeg"),
        ("gif", "image/gif"),
        ("svg", "image/svg+xml"),
        ("ico", "image/x-icon"),
        ("txt", "text/plain"),
    ]);
  Path::new(file_path)
    .extension()
    .and_then(|ext| ext.to_str())
    .and_then(|ext| mime_types.get(ext))
    .copied().unwrap_or("application/octet-stream")

}
