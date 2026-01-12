use std::io::{Write, stdin, stdout};

use rtils::{Exception, server::{serve, static_serve}, sscanf, throw, try_catch};

#[tokio::main]
async fn main(){
    static_serve("serve_dir".into(), "127.0.0.1:8080".to_owned()).await;
}