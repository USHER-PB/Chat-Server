use std::{ sync::{mpsc::{self, Receiver}, Arc, Mutex}, thread};

pub struct ThreadPool {
    pub worker: Vec<Worker>,
    pub sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
  pub  fn new(size: usize)-> Result<ThreadPool, & 'static str> {
        assert!(size > 0);

        let (sender, reciever) = mpsc::channel();

        let reciever = Arc::new(Mutex::new(reciever));

        let mut worker = Vec::with_capacity(size);

        for id in 0..size {
        worker.push(match Worker::new(id, Arc::clone(&reciever)){
               Ok(worker)=> worker,
               Err(e) => {
                  eprintln!("no worker found: {}", e);
                  continue;
               }
        
           })
           
         }
        Ok(ThreadPool {
            worker,
            sender: Some(sender),
        })
    }
  

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let senders = match &self.sender{
            Some(sender) => sender,
            None  => {
                eprintln!("no sender found ");
                return;
            }
        };
       match senders.send(job){
        Ok(job) => job,
        Err(e) => {
            eprintln!("no job send: {} ", e);
            return;
        }
       };
    }
   }

pub struct Worker {
    id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;
impl Worker {
   pub  fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>)-> Result< Worker,  &'static str>{
            let thread = thread::spawn(move || {
                loop {
                  let reciever =   match receiver.lock() {
                        Ok(val) => match val.recv(){
                        Ok(reciever) => reciever,
                        Err(_) => break,
                        },
                        Err(e) => {
                           eprintln!("No receiver found: {}", e);
                           break;            
                        }
                        
                    
                    };
                    println!("Worker {} got a job, executing...", id);
                    reciever();
                }
            });
        Ok(Worker{
         id,
         thread: Some(thread),
    })

   }
}


impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for opertaor in &mut self.worker {
            if let  Some(thread) = opertaor.thread.take() {
                match thread.join() {
                    Ok(_) => {println!("the sender was shutdown succesfully")},
                    Err(e) => {eprintln!("Not all Good: {:?}",e)},
                }
            }
        }
    }
}
