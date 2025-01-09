use std::{sync::{mpsc, Arc, Mutex}, thread};


type Job = Box<dyn FnOnce()+Send+'static>;

pub(crate) struct ThreadPool {
  workers:Vec<Worker>,
  sender:Option<mpsc::Sender<Job>>
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
    ThreadPool {
      workers:workers,
      sender:Some(sender),
    }
  }
  pub fn excute<F>(&self, f:F) where F:FnOnce()+Send+'static {
    let job = Box::new(f);
    self.sender.as_ref().unwrap().send(job).unwrap()
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
