use http::{http::HttpRequest, server::Server};

// use crate::http::server::Server;
mod http;
fn main() {

  let mut server = Server::new("7878");
  server.set_static("public");
  server.get("/name", get_name);
  server.run();
  // Server::start("7878");
  // let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
  // for stream in listener.incoming() {
  //   println!("connection etablished!");
  //   let stream = stream.unwrap();
  //   handle_stream(stream);
  // }
}

fn get_name(request:HttpRequest)->String{
  let method = request.method;
  let version = request.version;
  return format!("{method} {version} World");
}

