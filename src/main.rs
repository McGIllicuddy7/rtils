use std::io::stdin;

use rtils::{msg::{TestObject}, sscanf,dyn_scanf};
use rtils::msg::test_object_test_wrapper_direct;

#[tokio::main]
 async fn main(){
    
//file_server("serve_dir", "127.0.0.1:8080".to_string()).await;
    let mut x = TestObject{};
    test_object_test_wrapper_direct(&mut x, 1,2).unwrap();
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