use std::{sync::{mpsc, Arc, Mutex}, thread};

use tokio::runtime::Runtime;


type Job = Box<dyn FnOnce()+Send+'static>;

pub(crate) struct ThreadPool {
  workers:Vec<Worker>,
  sender:Option<mpsc::Sender<Job>>,
  runtime:Arc<Runtime>
}

impl ThreadPool {
  pub fn new(size:usize)->Self {
    assert!(size > 0); 
    let (sender, receiver) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));
    let mut workers = Vec::with_capacity(size);
    for id in 0..size {
      workers.push(Worker::new(id, Arc::clone(&receiver)));
    }
    let runtime = Arc::new(Runtime::new().unwrap());
    ThreadPool {
      workers:workers,
      sender:Some(sender),
      runtime:runtime
    }
  }
  pub fn excute<F>(&self, f:F) where F:FnOnce()+Send+'static {
    let job = Box::new(f);
    self.sender.as_ref().unwrap().send(job).unwrap()
  }
  pub fn excute_async<Fut>(&self, f:Fut) where Fut:std::future::Future<Output = ()> + Send + 'static {
    let runtime = Arc::clone(&self.runtime);
    self.excute(move || {
      runtime.spawn(f);
    });
  }
}

struct Worker {
  id:usize,
  thread:Option<thread::JoinHandle<()>>
}

impl Worker {
   fn new(id:usize, reciver:Arc<Mutex<mpsc::Receiver<Job>>>)->Worker{
    let thread = thread::spawn(move || loop {
      let message = reciver.lock().unwrap().recv();
      match message {
        Ok(job)=>{
          println!("Wroker {id}, got the job;");
          job();
        }
        Err(_)=>{
          println!("Worker {id} disconnected;");
          break;
        }

      }
    });
    Worker {id, thread:Some(thread)}

  } 
}
impl Drop for ThreadPool {
  fn drop(&mut self) {
    drop(self.sender.take());
    for worker in &mut self.workers {
      println!("shutting down the workers {}", worker.id);
      if let Some(thread) = worker.thread.take() {
        thread.join().unwrap();
      };
    }
  } 
}
