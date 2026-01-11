use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

#[allow(unused)]
pub type KeyCode = char;

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum MouseInput {
    LeftClick {
        x: i32,
        y: i32,
        is_currently_down: bool,
    },
    RightClick {
        x: i32,
        y: i32,
        is_currently_down: bool,
    },
    MiddleClick {
        x: i32,
        y: i32,
        is_currently_down: bool,
    },
    Move {
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        is_left_down: bool,
        is_right_down: bool,
        is_middle_down: bool,
    },
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub struct SubId {
    inner: u64,
}
impl SubId {
    pub fn invalid() -> Self {
        Self { inner: 0 }
    }
    pub fn is_valid(&self) -> bool {
        self.inner != 0
    }
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub struct ServiceId {
    inner: u64,
}
impl ServiceId {
    pub fn invalid() -> Self {
        Self { inner: 0 }
    }
    pub fn is_valid(&self) -> bool {
        self.inner != 0
    }
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub struct SourceId {
    inner: u64,
}
impl SourceId {
    pub fn invalid() -> Self {
        Self { inner: 0 }
    }
    pub fn is_valid(&self) -> bool {
        self.inner != 0
    }
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub struct TargetId {
    inner: u64,
}
impl TargetId {
    pub fn invalid() -> Self {
        Self { inner: 0 }
    }
    pub fn is_valid(&self) -> bool {
        self.inner != 0
    }
}

pub trait ThreadSafeIsh: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> ThreadSafeIsh for T {}

pub enum Event<T: ThreadSafeIsh> {
    CreateSubscriber(Box<dyn EventSub<T>>),
    DestroySubscriber {
        id: SubId,
    },
    CreateService(Box<dyn Service<T>>),
    DestroyService {
        id: ServiceId,
    },
    KeyBoardInput {
        key_code: KeyCode,
    },
    MouseInput {
        input_mouse_input: MouseInput,
    },
    TcpConnection {
        stream: TcpStream,
    },
    ExternalInput {
        source_id: SourceId,
        target_id: TargetId,
        input: Box<[u8]>,
    },
    UserDefined(T),
    RpcIo{
        io:crate::rpc::RpcMessage,
    }
}
impl<T: Clone + ThreadSafeIsh> Event<T> {
    pub fn try_clone(&self) -> Option<Event<T>> {
        match self {
            Event::CreateSubscriber(_) => None,
            Event::DestroySubscriber { id: _ } => None,
            Event::CreateService(_) => None,
            Event::DestroyService { id: _ } => None,
            Event::KeyBoardInput { key_code } => Some(Event::KeyBoardInput {
                key_code: *key_code,
            }),
            Event::MouseInput { input_mouse_input } => Some(Event::MouseInput {
                input_mouse_input: *input_mouse_input,
            }),
            Event::TcpConnection { stream: _ } => None,
            Event::ExternalInput {
                source_id,
                target_id,
                input,
            } => Some(Event::ExternalInput {
                source_id: *source_id,
                target_id: *target_id,
                input: input.clone(),
            }),
            Event::UserDefined(t) => Some(Event::UserDefined(t.clone())),
            Event::RpcIo {  io:_}=>None,
        }
    }

    pub fn is_clonable(&self) -> bool {
        match self {
            Event::CreateSubscriber(_) => false,
            Event::DestroySubscriber { id: _ } => false,
            Event::CreateService(_) => false,
            Event::DestroyService { id: _ } => false,
            Event::KeyBoardInput { key_code: _ } => true,
            Event::MouseInput {
                input_mouse_input: _,
            } => true,
            Event::TcpConnection { stream: _ } => false,
            Event::ExternalInput {
                source_id: _,
                target_id: _,
                input: _,
            } => false,
            Event::UserDefined(_) => true,
            Event::RpcIo { io:_ }=>false,
        }
    }
}

pub enum EventRequest {
    None,
    Shared,
    Owned,
}

#[async_trait]
pub trait EventSub<T: ThreadSafeIsh>: ThreadSafeIsh {
    async fn on_create(&mut self, self_id: SubId, sender: EventSync<T>);

    async fn wants_event(&self, event: &T) -> Result<EventRequest, Box<dyn Error>> {
        _ = event;
        Ok(EventRequest::None)
    }

    async fn wants_global_event(&self, event: &Event<T>) -> Result<EventRequest, Box<dyn Error>> {
        _ = event;
        Ok(EventRequest::None)
    }

    async fn on_event_owned(&mut self, event: T) -> Result<(), Box<dyn Error>> {
        _ = event;
        Ok(())
    }

    async fn on_event(&mut self, event: &T) -> Result<(), Box<dyn Error>>;

    async fn on_global_event<'a>(&'a self, event: &Event<T>) -> Result<(), Box<dyn Error>> {
        _ = event;
        Ok(())
    }

    async fn on_global_event_owned(&mut self, event: Event<T>) -> Result<(), Box<dyn Error>> {
        _ = event;
        Ok(())
    }
}

#[async_trait]
pub trait Service<T: ThreadSafeIsh>: ThreadSafeIsh {
    async fn create(&mut self, id: ServiceId, sender: EventSync<T>);
    async fn update(&mut self) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
pub trait Daemon<T:ThreadSafeIsh, >:ThreadSafeIsh{
    async fn run();
}
struct Handler<T: ThreadSafeIsh> {
    subscribers: BTreeMap<u64, Box<dyn EventSub<T>>>,
    services: BTreeMap<u64, Box<dyn Service<T>>>,
    daemons:BTreeMap<u64, Daemon>,
    sender: Sender<Event<T>>,
}

pub struct EventHandler<T: ThreadSafeIsh> {
    pub channel: Receiver<Event<T>>,
    handler: Handler<T>,
}

impl<T: ThreadSafeIsh> Handler<T> {
    fn new(sender: Sender<Event<T>>) -> Self {
        Self {
            subscribers: BTreeMap::new(),
            sender,
            services: BTreeMap::new(),
        }
    }

    pub async fn create_subscriber(
        &mut self,
        mut event_sub: Box<dyn EventSub<T>>,
        sender: Sender<Event<T>>,
    ) {
        let mut min = 0;
        for i in 0..=u64::MAX {
            if !self.subscribers.contains_key(&i) {
                min = i;
                break;
            }
        }
        if min == u64::MAX {
            panic!();
        }
        event_sub
            .as_mut()
            .on_create(SubId { inner: min }, EventSync::new(sender))
            .await;
        self.subscribers.insert(min, event_sub);
    }

    pub async fn destroy_subscriber(&mut self, id: SubId) {
        self.subscribers.remove(&id.inner);
    }

    pub async fn create_service(
        &mut self,
        mut service: Box<dyn Service<T>>,
        sender: Sender<Event<T>>,
    ) {
        let mut min = 0;
        for i in 0..=u64::MAX {
            if !self.services.contains_key(&i) {
                min = i;
                break;
            }
        }
        if min == u64::MAX {
            panic!();
        }
        service
            .create(ServiceId { inner: min }, EventSync::new(sender))
            .await;
        self.services.insert(min, service);
    }

    pub async fn destroy_service(&mut self, id: ServiceId) {
        self.services.remove(&id.inner);
    }

    pub async fn handle_user_event(&mut self, ev: T) -> Result<(), Box<dyn Error>> {
        for i in &mut self.subscribers {
            let v = i.1.wants_event(&ev).await?;
            match v {
                EventRequest::None => {
                    continue;
                }
                EventRequest::Shared => {
                    i.1.as_mut().on_event(&ev).await?;
                }
                EventRequest::Owned => {
                    i.1.as_mut().on_event_owned(ev).await?;
                    break;
                }
            }
        }
        Ok(())
    }

    pub async fn run_event(&mut self, i: Event<T>) -> Result<(), Box<dyn Error>> {
        match i {
            Event::CreateSubscriber(event_sub) => {
                self.create_subscriber(event_sub, self.sender.clone()).await;
            }
            Event::DestroySubscriber { id } => {
                self.destroy_subscriber(id).await;
            }
            Event::CreateService(service) => {
                self.create_service(service, self.sender.clone()).await;
            }
            Event::DestroyService { id } => {
                self.destroy_service(id).await;
            }
            Event::UserDefined(x) => {
                self.handle_user_event(x).await?;
            }
            _ => {
                todo!()
            }
        }
        Ok(())
    }
}

impl<T: ThreadSafeIsh> EventHandler<T> {
    pub fn new() -> (std::sync::mpsc::Sender<Event<T>>, Self) {
        let (sender, reciever) = std::sync::mpsc::channel();
        (
            sender.clone(),
            Self {
                channel: reciever,
                handler: Handler::new(sender),
            },
        )
    }

    pub async fn handle_events(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let ev = self.channel.try_recv();
            match ev {
                Ok(ev) => {
                    self.handler.run_event(ev).await?;
                }
                Err(e) => match e {
                    std::sync::mpsc::TryRecvError::Empty => {
                        break;
                    }
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        return Err("disconnected".into());
                    }
                },
            }
        }
        Ok(())
    }

    pub async fn handle_services(&mut self) -> Result<(), Box<dyn Error>> {
        for service in self.handler.services.values_mut() {
            service.as_mut().update().await?;
        }
        Ok(())
    }

    pub async fn run(&mut self, setup: impl AsyncFn(EventSync<T>)) {
        setup(EventSync::new(self.handler.sender.clone())).await;
        loop {
            let res = self.handle_events().await;
            if res.is_err() {
                todo!();
            }
            let res = self.handle_services().await;
            if res.is_err() {
                todo!()
            }
        }
    }
}

pub struct EventSync<T: ThreadSafeIsh> {
    sender: Option<Sender<Event<T>>>,
}
impl<T: ThreadSafeIsh> EventSync<T> {
    pub fn new(sender: Sender<Event<T>>) -> Self {
        Self {
            sender: Some(sender),
        }
    }

    pub fn new_event(&self, ev: T) -> Result<(), Box<dyn Error>> {
        self.sender
            .as_ref()
            .unwrap()
            .send(Event::UserDefined(ev))
            .unwrap();
        Ok(())
    }

    pub fn new_event_global(&self, ev:Event<T>) -> Result<(), Box<dyn Error>> {
        self.sender
            .as_ref()
            .unwrap()
            .send(ev)
            .unwrap();
        Ok(())
    }

    pub async fn create_new_subscriber<Sub: EventSub<T> + 'static>(
        &self,
        subscriber: Sub,
    ) -> Result<(), Box<dyn Error>> {
        let bx = Box::new(subscriber) as Box<dyn EventSub<T>>;
        self.sender
            .as_ref()
            .unwrap()
            .send(Event::CreateSubscriber(bx))?;
        Ok(())
    }

    pub fn invalid() -> Self {
        Self { sender: None }
    }
}

pub struct BPipe<T> {
    sending: Arc<Mutex<VecDeque<T>>>,
    recieving: Arc<Mutex<VecDeque<T>>>,
    done: Arc<AtomicBool>,
}
impl<T> BPipe<T> {
    pub fn create() -> (BPipe<T>, BPipe<T>) {
        let p1 = Arc::new(Mutex::new(VecDeque::new()));
        let p2 = Arc::new(Mutex::new(VecDeque::new()));
        let done = Arc::new(AtomicBool::new(false));
        let out1 = BPipe {
            sending: p1.clone(),
            recieving: p2.clone(),
            done: done.clone(),
        };
        let out2 = BPipe {
            sending: p2,
            recieving: p1,
            done,
        };
        (out1, out2)
    }

    pub fn send(&self, v: T) -> Result<(), Box<dyn Error>> {
        if self.done.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("done".into());
        }
        let mut sending = self.sending.lock().unwrap();
        sending.push_back(v);
        Ok(())
    }

    pub fn recieve(&self) -> Result<Option<T>, Box<dyn Error>> {
        if self.done.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("done".into());
        }
        let mut recieving = self.recieving.lock().unwrap();
        Ok(recieving.pop_front())
    }

    pub fn recieve_wait(&self) -> Result<T, Box<dyn Error>> {
        loop {
            if self.done.load(std::sync::atomic::Ordering::Relaxed) {
                return Err("done".into());
            }
            let mut recieving = self.recieving.lock().unwrap();
            if let Some(x) = recieving.pop_front() {
                return Ok(x);
            }
        }
    }

    pub fn recieve_async(&self) -> impl Future<Output = Result<T, Box<dyn Error>>> {
        pub struct Out<T> {
            reciever: Arc<Mutex<VecDeque<T>>>,
            done: Arc<AtomicBool>,
        }
        impl<T> Future for Out<T> {
            type Output = Result<T, Box<dyn Error>>;
            fn poll(
                self: std::pin::Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                if self.done.load(std::sync::atomic::Ordering::Relaxed) {
                    return std::task::Poll::Ready(Err("done".into()));
                }
                let tmp = self.reciever.try_lock();
                match tmp {
                    Ok(mut x) => {
                        if let Some(x) = x.pop_front() {
                            std::task::Poll::Ready(Ok(x))
                        } else {
                            std::task::Poll::Pending
                        }
                    }
                    Err(er) => match er {
                        std::sync::TryLockError::Poisoned(poison_error) => {
                            let mut e = poison_error.into_inner();
                            if let Some(x) = e.pop_front() {
                                std::task::Poll::Ready(Ok(x))
                            } else {
                                std::task::Poll::Pending
                            }
                        }
                        std::sync::TryLockError::WouldBlock => std::task::Poll::Pending,
                    },
                }
            }
        }
        Out {
            reciever: self.recieving.clone(),
            done: self.done.clone(),
        }
    }
}
impl<T> Drop for BPipe<T> {
    fn drop(&mut self) {
        self.done.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

pub struct EventForwarder<T: ThreadSafeIsh> {
    self_id: SubId,
    sender: EventSync<T>,
    pipe: Option<BPipe<T>>,
    should_forward: Box<dyn Fn(&T) -> bool + Send + Sync + 'static>,
    event_pipe: Option<BPipe<Event<T>>>,
    should_forward_event: Box<dyn Fn(&Event<T>) -> bool + Send + Sync + 'static>,
}
impl<T: ThreadSafeIsh + Clone> EventForwarder<T> {
    pub async fn new(
        should_forward: impl Fn(&T) -> bool + Send + Sync + 'static,
        sync: EventSync<T>,
    ) -> BPipe<T> {
        let (this_pipe, out_pipe) = BPipe::create();
        fn f_false<T: ThreadSafeIsh>(_: &Event<T>) -> bool {
            false
        }
        let this = Self {
            pipe: Some(this_pipe),
            should_forward: Box::new(should_forward),
            event_pipe: None,
            should_forward_event: Box::new(f_false),
            self_id: SubId::invalid(),
            sender: EventSync::invalid(),
        };
        sync.create_new_subscriber(this).await.unwrap();
        out_pipe
    }

    pub async fn new_globals(
        should_forward: impl Fn(&Event<T>) -> bool + Send + Sync + 'static,
        sync: EventSync<T>,
    ) -> BPipe<Event<T>> {
        let (this_pipe, out_pipe) = BPipe::create();
        fn f_false<T: ThreadSafeIsh>(_: &T) -> bool {
            false
        }
        let this = Self {
            pipe: None,
            should_forward: Box::new(f_false),
            event_pipe: Some(this_pipe),
            should_forward_event: Box::new(should_forward),
            self_id: SubId::invalid(),
            sender: EventSync::invalid(),
        };
        sync.create_new_subscriber(this).await.unwrap();
        out_pipe
    }

    pub async fn new_all(
        should_forward: impl Fn(&T) -> bool + Send + Sync + 'static,
        should_forward_globals: impl Fn(&Event<T>) -> bool + Send + Sync + 'static,
        sync: EventSync<T>,
    ) -> (BPipe<T>, BPipe<Event<T>>) {
        let (this_pipe, out_pipe) = BPipe::create();
        let (this_events, out_events) = BPipe::create();
        let this = Self {
            pipe: Some(this_pipe),
            should_forward: Box::new(should_forward),
            event_pipe: Some(this_events),
            should_forward_event: Box::new(should_forward_globals),
            self_id: SubId::invalid(),
            sender: EventSync::invalid(),
        };
        sync.create_new_subscriber(this).await.unwrap();
        (out_pipe, out_events)
    }
}

#[async_trait]
impl<T: ThreadSafeIsh + Clone> EventSub<T> for EventForwarder<T> {
    async fn on_create(&mut self, self_id: SubId, sender: EventSync<T>) {
        self.self_id = self_id;
        self.sender = sender;
    }

    async fn on_event(&mut self, event: &T) -> Result<(), Box<dyn Error>> {
        self.pipe.as_ref().unwrap().send(event.clone())
    }

    async fn on_global_event<'a>(&'a self, event: &Event<T>) -> Result<(), Box<dyn Error>> {
        let Some(event) = event.try_clone() else {
            return Ok(());
        };
        self.event_pipe.as_ref().unwrap().send(event)?;
        Ok(())
    }

    async fn wants_global_event(&self, event: &Event<T>) -> Result<EventRequest, Box<dyn Error>> {
        if self.event_pipe.is_none() {
            return Ok(EventRequest::None);
        }
        if event.is_clonable() {
            if (self.should_forward_event)(event) {
                Ok(EventRequest::Shared)
            } else {
                Ok(EventRequest::None)
            }
        } else {
            Ok(EventRequest::None)
        }
    }

    async fn wants_event(&self, event: &T) -> Result<EventRequest, Box<dyn Error>> {
        if (self.should_forward)(event) {
            Ok(EventRequest::Shared)
        } else {
            Ok(EventRequest::None)
        }
    }
}

pub struct IdAllocator {
    al: Mutex<IdAllocatorInternal>,
}
impl IdAllocator {
    pub fn alloc_target(&self) -> TargetId {
        self.al.lock().unwrap().alloc_target()
    }

    pub fn alloc_source(&self) -> SourceId {
        self.al.lock().unwrap().alloc_source()
    }

    pub fn alloc_general(&self) -> u64 {
        self.al.lock().unwrap().alloc_general()
    }

    pub fn free_target(&self, id: TargetId) {
        self.al.lock().unwrap().free_target(id)
    }

    pub fn free_source(&self, id: SourceId) {
        self.al.lock().unwrap().free_source(id)
    }

    pub fn free_general(&self, id: u64) {
        self.al.lock().unwrap().free_general(id)
    }
}

pub static IDS: IdAllocator = IdAllocator {
    al: Mutex::new(IdAllocatorInternal::new()),
};

struct IdAllocatorInternal {
    target_ids: BTreeSet<u64>,
    source_ids: BTreeSet<u64>,
    general_ids: BTreeSet<u64>,
}
impl IdAllocatorInternal {
    pub const fn new() -> Self {
        Self {
            general_ids: BTreeSet::new(),
            source_ids: BTreeSet::new(),
            target_ids: BTreeSet::new(),
        }
    }

    pub fn alloc_source(&mut self) -> SourceId {
        let mut min = 1;
        for i in 1..=u64::MAX {
            min = i;
            if !self.source_ids.contains(&i) {
                break;
            }
        }
        if min == u64::MAX {
            panic!()
        }
        self.source_ids.insert(min);
        SourceId { inner: min }
    }

    pub fn alloc_target(&mut self) -> TargetId {
        let mut min = 1;
        for i in 1..=u64::MAX {
            min = i;
            if !self.target_ids.contains(&i) {
                break;
            }
        }
        if min == u64::MAX {
            panic!()
        }
        self.target_ids.insert(min);
        TargetId { inner: min }
    }

    pub fn alloc_general(&mut self) -> u64 {
        let mut min = 1;
        for i in 1..=u64::MAX {
            min = i;
            if !self.general_ids.contains(&i) {
                break;
            }
        }
        if min == u64::MAX {
            panic!()
        }
        self.general_ids.insert(min);
        min
    }

    pub fn free_source(&mut self, id: SourceId) {
        self.source_ids.remove(&id.inner);
    }

    pub fn free_target(&mut self, id: TargetId) {
        self.target_ids.remove(&id.inner);
    }

    pub fn free_general(&mut self, id: u64) {
        self.general_ids.remove(&id);
    }
}
