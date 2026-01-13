use std::io::{Write, stdin, stdout};

use rtils::{dyn_scanf, events::EventHandler, server::file_server, sscanf};


#[tokio::main]
 async fn main(){
    file_server("serve_dir", "127.0.0.1:8080".to_string()).await;
}
#[allow(unused)]
pub fn scanf_test(){
    let mut age = 0;
    let mut name = String::new();
    let mut buf = String::new();
    let i0 = stdin().read_line(&mut buf).unwrap();
    let success = sscanf!(buf, "{}, {}", age, name);
    println!("age:{}, name:{}", age, name);
}