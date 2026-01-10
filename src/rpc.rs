use std::{collections::{BTreeMap}, error::Error,sync::{Arc, Mutex}};
use async_trait::async_trait;

pub use crate::events::*;
pub struct WriteOnce<T>{
    v:Arc<Mutex<Option<T>>>,
}
impl<T> WriteOnce<T>{
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
    pub in_flight:BTreeMap<TargetId,RemoteProcedureCall>,
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