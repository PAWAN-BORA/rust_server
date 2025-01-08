use std::{collections::HashMap, fmt::Display};


pub enum HttpMethod {
  GET,
  POST,
  PUT,
  PATCH,
  DELETE,
  OPTIONS,
  HEAD,
  OTHER(String)
}

impl HttpMethod {
  pub fn from_str(method:&str)->Self {
    match method {
      "GET" => HttpMethod::GET,
      "POST" => HttpMethod::POST,
      "PUT" => HttpMethod::PUT,
      "PATCH" => HttpMethod::PATCH,
      "DELETE" => HttpMethod::DELETE,
      "OPTIONS" => HttpMethod::OPTIONS,
      "HEAD" => HttpMethod::HEAD,
      _ => HttpMethod::OTHER(method.to_string()),
        
    }
  }
}
impl Display for HttpMethod {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
           HttpMethod::GET => write!(f, "GET"),
           HttpMethod::POST=> write!(f, "POST"),
           HttpMethod::PUT => write!(f, "PUT"),
           HttpMethod::PATCH => write!(f, "PATCH"),
           HttpMethod::DELETE => write!(f, "DELETE"),
           HttpMethod::OPTIONS => write!(f, "OPTIONS"),
           HttpMethod::HEAD => write!(f, "HEAD"),
           HttpMethod::OTHER(method) => write!(f, "{method}"),
        }
    } 
}
pub struct HttpRequest {
  pub method:HttpMethod,
  pub version:String,
  pub path:String,
  pub header:HashMap<String, String>,
  pub body:Option<String>
}

pub enum HttpStatusCode{
  Success = 200,
  Created = 201,
  NoContent = 204,
  PartialContent= 206,
  BadRequest = 400,
  Unauthorized = 401,
  Forbidden = 403,
  NotFound = 404,
  MethodNotAllowed = 405,
  NotAcctable = 406,
  Conflict = 409,
  InternalServerError = 500,
  NotImplemented = 501,
  BadGatway = 502,
  ServiceUnavailable = 503,
}
impl Display for HttpStatusCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self {
      HttpStatusCode::Success => write!(f, "200 Ok"),
      HttpStatusCode::Created => write!(f, "201 Created"),
      HttpStatusCode::NoContent => write!(f, "204 No Content"),
      HttpStatusCode::PartialContent => write!(f, "206 Partial Content"),
      HttpStatusCode::BadRequest => write!(f, "400 Bad Request"),
      HttpStatusCode::Unauthorized=> write!(f, "401 Unauthorized"),
      HttpStatusCode::Forbidden => write!(f, "403 Forbidden"),
      HttpStatusCode::NotFound=> write!(f, "404 Not Found"),
      HttpStatusCode::MethodNotAllowed=> write!(f, "405 Method Not Allowed"),
      HttpStatusCode::NotAcctable => write!(f, "406 Not Acceptable"),
      HttpStatusCode::Conflict=> write!(f, "409 Conflict"),
      HttpStatusCode::InternalServerError => write!(f, "500 Internal Server Error"),
      HttpStatusCode::NotImplemented=> write!(f, "501 Not Implemented"),
      HttpStatusCode::BadGatway=> write!(f, "502 Bad Gateway"),
      HttpStatusCode::ServiceUnavailable=> write!(f, "503 Service Unavailable"),
    }
  } 
}
pub struct HttpResponse {
  pub status:HttpStatusCode,
  pub header:HashMap<String, String>,
  pub body:Option<String>
}

impl HttpResponse {
  pub fn new()->Self {
    let header = HashMap::from([
      ("Content-Type".to_string(), "text/html".to_string()),
      ("Connection".to_string(), "keep-alive".to_string()),
      ("X-Powered-By".to_string(), "rust_server".to_string()),
    ]);
    HttpResponse {
      status:HttpStatusCode::Success,
      header:header,
      body:None,
    }
  }
  pub fn send(&mut self, body:String) {
    self.body = Some(body);
  }
  pub fn set_header(&mut self, key:String, value:String){
    self.header.insert(key, value);
  }
}

