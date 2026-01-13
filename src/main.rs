use std::io::{Write, stdin, stdout};

use rtils::{sscanf, dyn_scanf};


 fn main(){

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