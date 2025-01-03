use std::{thread, time::Duration};

use http::{http::{HttpRequest, HttpResponse}, server::Server};

// use crate::http::server::Server;
mod http;
fn main() {

  let mut server = Server::new("7878");
  server.set_static("public");
  server.get("/name", get_name);
  server.get("/sleep", hold_server);
  server.run();
  // Server::start("7878");
  // let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
  // for stream in listener.incoming() {
  //   println!("connection etablished!");
  //   let stream = stream.unwrap();
  //   handle_stream(stream);
  // }
}

fn get_name(request:HttpRequest, response:&mut HttpResponse){
  let method = request.method;
  let version = request.version;
  response.send(format!("{method} {version} World"));
}

fn hold_server(request:HttpRequest, response:&mut HttpResponse){
  let method = request.method;
  let path = request.path;
  thread::sleep(Duration::from_secs(10));
  response.send(format!("{method} {path} waiting for 5 sec"));
}

