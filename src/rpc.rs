
use std::{collections::BTreeMap, error::Error, sync::{Arc, Mutex}};
use async_trait::async_trait;
use tokio::net::TcpStream;

pub use crate::events::*;
pub struct WriteOnce<T>{
    v:Arc<Mutex<Option<T>>>,
}
impl<T> WriteOnce<T>{
    pub fn create()->(Self, Self){
        let vout = Arc::new(Mutex::new(None));
        let a = Self{v:vout.clone()};
        let b = Self{v:vout};
        (a,b)
    }
    pub fn read(&self)->impl Future<Output =Result<T, Box<dyn Error>>>{
        struct Out<T>{
            v:Arc<Mutex<Option<T>>>
        }
        impl<T> Future for Out<T>{
            type Output = Result<T, Box<dyn Error>>;
            fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
                let lock = self.v.try_lock();
                match lock{
                    Ok(mut t) => {
                        if let Some(m) = t.take(){
                            std::task::Poll::Ready(Ok(m))
                        }else{
                            std::task::Poll::Pending
                        }
               
                    },
                    Err(e) => {
                        match e{
                            std::sync::TryLockError::Poisoned(poison_error) =>{
                                let mut lck = poison_error.into_inner();
                                if let Some(m) = lck.take(){
                                    std::task::Poll::Ready(Ok(m))
                                }else{
                                    std::task::Poll::Pending
                                }
                            }
                            std::sync::TryLockError::WouldBlock => {
                                std::task::Poll::Pending 
                            }
                        }
                    }
                }
            }
        }
        let out = Out{
            v:self.v.clone()
        };
        out
    }

    pub fn write(self, v:T){
        let lock = self.v.lock();
        let mut value = match lock{
            Ok(value)=>{
                value
            }
            Err(value)=>{
                value.into_inner()
            }
        };
        *value = Some(v);
    }

    pub fn try_read(&self)->Result<Option<T>, Box<dyn Error>>{
              let lock = self.v.try_lock();
                match lock{
                    Ok(mut t) => {
                        if let Some(m) = t.take(){
                            Ok(Some(m))
                        }else{
                            Ok(None)
                        }
               
                    },
                    Err(e) => {
                        match e{
                            std::sync::TryLockError::Poisoned(poison_error) =>{
                                let mut lck = poison_error.into_inner();
                                if let Some(m) = lck.take(){
                                    Ok(Some(m))
                                }else{
                                    Ok(None)
                                }
                            }
                            std::sync::TryLockError::WouldBlock => {
                                Ok(None)
                            }
                        }
                    }
                }
    }

}

pub struct RemoteProcedureCall{
    pub id:TargetId,
    pub source:SourceId, 
    pub to_write_to:WriteOnce<Box<[u8]>> 
}

pub struct RPCDaemon{
    pub in_flight:BTreeMap<u64,RemoteProcedureCall>,
    pub streams:BTreeMap<u64, TcpStream>,
}

#[async_trait]
impl<T: ThreadSafeIsh> Service<T> for RPCDaemon{
    async fn create(&mut self, id: ServiceId, sender: EventSync<T>) {
        todo!()
    }
    async fn update(&mut self) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
pub async fn call_with_response<T:ThreadSafeIsh>(events:&EventSync<T>,source:RpcConnectionId, data:Vec<u8>)->Result<Vec<u8>, Box<dyn Error>>{
        let id = IDS.alloc_target();
        let (input, output) = WriteOnce::create();
        let msg = RpcMessage::Rpc{ input: data, output: output, target_id: id, id: source};
        let ev = Event::RpcIo { io: msg };
        events.new_event_global(ev)?;
        let out =input.read().await?;
        Ok(out)
}

pub fn rpc_connect<T:ThreadSafeIsh>(events:&EventSync<T>,stream:TcpStream)->RpcConnectionId{
    let id = IDS.alloc_general();
    let con = RpcConnectionId { internal: id };
    let ev = Event::RpcIo { io: RpcMessage::RpcConnectionCreate { stream, id: con } };
    events.new_event_global(ev).unwrap();
    con
}
pub fn rpc_disconnect<T:ThreadSafeIsh>(events:&EventSync<T>,id:RpcConnectionId){
    let ev = Event::RpcIo { io: RpcMessage::RpcConnectionDestroy { id} };
    events.new_event_global(ev).unwrap();
}
pub fn send_message<T:ThreadSafeIsh>(events:&EventSync<T>, id:RpcConnectionId, message:Vec<u8>){
    let ev = Event::RpcIo { io: RpcMessage::GeneralMessageSend { input: message, id } };
    events.new_event_global(ev).unwrap();
}
pub async fn recieve_message<T:ThreadSafeIsh>(events:&EventSync<T>, id:RpcConnectionId)->Result<Vec<u8>, Box<dyn Error>>{
    let (input,output) = WriteOnce::<Vec<u8>>::create();
    let msg = RpcMessage::GeneralMessageRecieve { output, id};
    let ev = Event::RpcIo { io: msg };
    events.new_event_global(ev)?;
    let x = input.read().await?;
    Ok(x)
}

#[derive(Clone, Copy)]
pub struct RpcConnectionId{
    internal:u64
}
impl RpcConnectionId{
    pub fn invalid()->Self{
        Self { internal: 0 }
    }
    pub fn is_valid(&self)->bool{
        self.internal != 0
    }
}
pub enum RpcMessage{
    RpcConnectionCreate{
        stream:TcpStream,
        id:RpcConnectionId,
    },
    Rpc{
        input:Vec<u8>,
        output:WriteOnce<Vec<u8>>,
        target_id:TargetId,
        id:RpcConnectionId,
    },
    GeneralMessageSend{
        input:Vec<u8>, 
        id:RpcConnectionId,
    },
    GeneralMessageRecieve{
        output:WriteOnce<Vec<u8>>,
        id:RpcConnectionId,
    }, 
    RpcConnectionDestroy{
        id:RpcConnectionId,
    }
}
#[repr(C)]
pub struct RpcHeader{
    read_size:u64,
    target:u64,
}