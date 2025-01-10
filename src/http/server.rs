use std::{collections::HashMap, net::TcpListener, sync::{Arc, RwLock}, thread};

use super::{handle_stream::HandleStream, http::{HttpRequest, HttpResponse}, thread_pool::{self, ThreadPool}};


pub(crate) type RouteFn = fn(HttpRequest, &mut HttpResponse);
type Routes = HashMap<String, RouteFn>;
pub struct Server {
  pub port:String,
  pub public:Option<String>,
  pub get_routes:Routes,
  pub post_routes:Routes,
  pub delete_routes:Routes,
  pub put_routes:Routes,
  pub thread_num:usize,
  
}

impl Server {
  pub fn new(port:&str)->Self{
    return Server {
      port:port.to_string(),
      public:None,
      get_routes:HashMap::new(),
      post_routes:HashMap::new(),
      thread_num:4,
      delete_routes:HashMap::new(),
      put_routes:HashMap::new(),
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
  pub fn put(&mut self, path:&str, fun:RouteFn) {
    self.put_routes.insert(path.to_string(), fun);
  }
  pub fn delete(&mut self, path:&str, fun:RouteFn) {
    self.delete_routes.insert(path.to_string(), fun);
  }
  pub fn run(self) {
    let addr = format!("127.0.0.1:{}", &self.port);
    let listener = TcpListener::bind(addr).unwrap();
    let thread_pool = ThreadPool::new(self.thread_num);
    let server = Arc::new(RwLock::new(self));
    for stream in listener.incoming() {
      let server = Arc::clone(&server);
      thread_pool.excute(move ||{
        match stream {
          Ok(stream)=>{
            HandleStream::new(stream, server).parse();
          },
          Err(err)=> {println!("Error in Stream: {}", err);}
        }
      });
    }
    println!("Shutting down.");
  }
}

