use std::sync::mpsc::Sender;
use std::net::SocketAddr;
use serde::de::Error;
use crate::{BadOption, dyn_scanf, get_extension, http_write_html_response, http_write_jpg_response, http_write_js_response, http_write_png_response, http_write_string_response};
use tokio::{io::AsyncReadExt, net::{TcpListener, TcpStream}};

use crate::{Throw, Throws, events::{Event, EventHandler}, sscanf};
pub enum HttpEvent{

}
pub async fn static_serve(dir:String, addr:String){
    let stream = TcpListener::bind(addr).await.unwrap();
    let (sender,mut context) = EventHandler::<HttpEvent>::new();
    let d1 = dir.clone();
    tokio::task::spawn(async move{listen(d1,stream, sender).await});
    context.run(async |_|{
        println!("called");
    }).await;
}

pub async fn listen(dir:String,stream:TcpListener, _sender:Sender<Event<HttpEvent>>)->Throws<()>{
    println!("listening");
    loop{
        println!("connected");
        let d1 = dir.clone();
        let (stream, addr) = stream.accept().await?;
        tokio::task::spawn(async move{
            let e = serve(d1, stream, addr).await;
            if let Err(e) =  e{
                //println!("e:{}", e);
            }
        });
    }
}


pub async fn serve(dir:String, mut stream:TcpStream, _addr:SocketAddr)->Throws<()>{
    let mut x = [0;4096];
    let x_l = stream.read(&mut x).await?;
    let st = str::from_utf8(&x[0..x_l])?;
    let l = st.lines().next().throw()?;
    let mut req_str = String::new();
    if l == "GET / HTTP/1.1"{
        req_str = "index.html".into();
    } else{
        let rs = sscanf!( l.to_string(),"GET /{} HTTP/1.1", req_str);
        if !rs{
            println!("{:#?}", l);
            return Err(BadOption{}.into());
        }
    }

    let d = dir.clone()+"/"+&req_str;
    let f = if let Ok(s) = std::fs::read(&d){
        s
    }else{
        println!("no such file or directory:{}",d);
        "error 404".as_bytes().to_vec()
    };
    let ext = get_extension(&d);
    match ext{
        ""=>{
            http_write_string_response(&mut stream, &f).await?;
        }
        "txt"=>{
            http_write_string_response(&mut stream, &f).await?;
        }
        "html"=>{
            http_write_html_response(&mut stream, &f).await?;
        }
        "png"=>{
            http_write_png_response(&mut stream, &f).await?;
        }
        "jpg"=>{
            http_write_jpg_response(&mut stream, &f).await?;
        }
        "jpeg"=>{
            http_write_jpg_response(&mut stream, &f).await?; 
        }
        "js"=>{
            http_write_js_response(&mut stream, &f).await?;  
        }
        _=>{
            todo!()
        }
    };
    Ok(())
}