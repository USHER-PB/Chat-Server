
use threadpool::{ ThreadPool, Worker};

mod threadpool;
mod connection;
use connection::connection;
use std::sync::Arc;
use std::sync::Mutex;
use std::net::TcpListener;

 fn main(){

    let messages = Arc::new(Mutex::new(Vec::new()));
    let listener = match TcpListener::bind("192.168.255.26:8789"){
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("no listener found  {}", e);
            return;
        }
    };
       
    let  operator = match ThreadPool::new(6){
        Ok(operator) => operator,
        Err(e) => {
            eprintln!("no threadpool created {}", e);
            return
        }
    };

      for stream in listener.incoming(){
        let messages = Arc::clone(&messages);
      let stream = match stream{
          Ok(stream) => stream,
          Err(e) => {
            eprintln!("no connection establish:{}", e);
            return
          }
      };

      operator.execute(move || {
        connection(stream, messages);
      });
}

}



