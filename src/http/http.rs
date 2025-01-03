use std::collections::HashMap;

pub struct HttpRequest {
  pub method:String,
  pub version:String,
  pub path:String,
  pub header:HashMap<String, String>,
  pub body:Option<String>
}

