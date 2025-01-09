use std::{thread, time::Duration};
use http::{http::{HttpRequest, HttpResponse}, server::Server};
mod http;
fn main() {
  let mut server = Server::new("7878");
  server.set_static("public");
  server.get("/name", get_name);
  server.get("/sleep", hold_server);
  server.post("/send", send_json);
  server.run();
}

fn get_name(request:HttpRequest, response:&mut HttpResponse){
  // let body = request.body_as_vec().unwrap();
  let method = request.method;
  let version = request.version;
  response.send(format!("{method} {version} World"));
}

fn send_json(request:HttpRequest, response:&mut HttpResponse){
  let body = request.body_as_str();
  response.send(body.to_string());
}
fn hold_server(request:HttpRequest, response:&mut HttpResponse){
  println!("{:?}", request.params);
  let method = request.method;
  let path = request.path;
  response.set_header("Access-Control-Allow-Origin".to_string(), "*".to_string());
  response.set_header("Content-Security-Policy".to_string(), "connect-src 'self' *".to_string());
  thread::sleep(Duration::from_secs(2));
  response.send(format!("{method} {path} waiting for 20 sec"));
}

