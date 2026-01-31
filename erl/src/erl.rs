use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    io::{Read, Write},
    net::TcpStream,
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
    task::{Context, Poll, Waker},
};

use serde::{Serialize, de::DeserializeOwned};

#[derive(Hash, PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy)]

pub struct ProcessId {
    id: u64,
}

impl ProcessId {
    pub fn invalid() -> Self {
        Self { id: 0 }
    }
    pub fn get(&self) -> u64 {
        self.id
    }
}

pub trait ProcessBase: Send {
    fn async_update(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>>;
    fn setup(&mut self, this_id: ProcessId, handle: RuntimeHandle);
}
pub trait Process: 'static {
    fn update(&mut self) -> impl std::future::Future<Output = ()> + Send;
    fn create(&mut self, this_id: ProcessId, handle: RuntimeHandle);
}
impl<T: Process + Send> ProcessBase for T {
    fn async_update(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        Box::pin(self.update())
    }
    fn setup(&mut self, this_id: ProcessId, handle: RuntimeHandle) {
        self.create(this_id, handle);
    }
}

pub struct Proc {
    pub state: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    pub handle: RuntimeHandleInner,
}

pub enum Message {
    ProcessExit,
    Message {
        to: ProcessId,
        from: ProcessId,
        data: Arc<[u8]>,
    },
    SpawnProcess(Box<dyn ProcessBase>),
    SpawnProcessNamed(Box<dyn ProcessBase>, String),
    ReturnedId(ProcessId),
    ProcessDoesNotExist,
    RequestNamedProcess(String),
    KillProcess(ProcessId),
    GetNamedProcess(String),
    DoesProcessExist(ProcessId),
}
pub struct RuntimeHandleInner {
    sending: Arc<Mutex<VecDeque<Message>>>,
    recieving: Arc<Mutex<VecDeque<Message>>>,
    request: Arc<Mutex<Option<(bool, Message)>>>,
    response: Arc<Mutex<Option<(bool, Message)>>>,
    invalid: bool,
}
impl RuntimeHandleInner {
    pub fn invalid() -> Self {
        Self {
            sending: Arc::new(Mutex::new(VecDeque::new())),
            recieving: Arc::new(Mutex::new(VecDeque::new())),
            response: Arc::new(Mutex::new(None)),
            request: Arc::new(Mutex::new(None)),
            invalid: true,
        }
    }
    pub fn new() -> (Self, Self) {
        let q1 = Arc::new(Mutex::new(VecDeque::new()));
        let q2 = Arc::new(Mutex::new(VecDeque::new()));
        let r1 = Arc::new(Mutex::new(None));
        let r2 = Arc::new(Mutex::new(None));

        (
            Self {
                sending: q1.clone(),
                recieving: q2.clone(),
                request: r1.clone(),
                response: r2.clone(),
                invalid: false,
            },
            Self {
                sending: q2,
                recieving: q1,
                request: r2,
                response: r1,
                invalid: false,
            },
        )
    }

    pub fn send(&self, msg: Message) -> impl Future<Output = ()> {
        assert!(!self.invalid);
        struct Fut<'a> {
            slf: &'a RuntimeHandleInner,
            msg: Option<Message>,
        }
        impl<'a> Future for Fut<'a> {
            type Output = ();

            fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                match self.slf.sending.try_lock() {
                    Ok(mut lock) => {
                        let msg = self.msg.take().unwrap();
                        lock.push_back(msg);
                        Poll::Ready(())
                    }
                    Err(es) => match es {
                        std::sync::TryLockError::Poisoned(poison_error) => {
                            let mut lock = poison_error.into_inner();
                            let msg = self.msg.take().unwrap();
                            lock.push_back(msg);
                            Poll::Ready(())
                        }
                        std::sync::TryLockError::WouldBlock => Poll::Pending,
                    },
                }
            }
        }
        Fut {
            slf: self,
            msg: Some(msg),
        }
    }

    pub fn receive(&self) -> impl Future<Output = Option<Message>> {
        struct Fut<'a> {
            slf: &'a RuntimeHandleInner,
        }
        impl<'a> Future for Fut<'a> {
            type Output = Option<Message>;

            fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                match self.slf.recieving.try_lock() {
                    Ok(mut lock) => Poll::Ready(lock.pop_front()),
                    Err(es) => match es {
                        std::sync::TryLockError::Poisoned(poison_error) => {
                            let mut lock = poison_error.into_inner();
                            Poll::Ready(lock.pop_front())
                        }
                        std::sync::TryLockError::WouldBlock => Poll::Pending,
                    },
                }
            }
        }
        Fut { slf: self }
    }

    pub fn receive_wait(&self) -> impl Future<Output = Message> {
        struct Fut<'a> {
            slf: &'a RuntimeHandleInner,
        }
        impl<'a> Future for Fut<'a> {
            type Output = Message;

            fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                match self.slf.recieving.try_lock() {
                    Ok(mut lock) => {
                        if let Some(out) = lock.pop_front() {
                            Poll::Ready(out)
                        } else {
                            Poll::Pending
                        }
                    }
                    Err(es) => match es {
                        std::sync::TryLockError::Poisoned(poison_error) => {
                            let mut lock = poison_error.into_inner();
                            if let Some(out) = lock.pop_front() {
                                Poll::Ready(out)
                            } else {
                                Poll::Pending
                            }
                        }
                        std::sync::TryLockError::WouldBlock => Poll::Pending,
                    },
                }
            }
        }
        Fut { slf: self }
    }

    pub fn recieve_request(&self) -> impl Future<Output = Option<Message>> {
        struct Fut<'a> {
            slf: &'a RuntimeHandleInner,
        }
        impl<'a> Future for Fut<'a> {
            type Output = Option<Message>;

            fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                match self.slf.response.try_lock() {
                    Ok(mut lock) => {
                        if lock.is_none() {
                            Poll::Ready(None)
                        } else {
                            if lock.as_ref().unwrap().0 {
                                Poll::Ready(None)
                            } else {
                                let lck = lock.take().unwrap();
                                Poll::Ready(Some(lck.1))
                            }
                        }
                    }
                    Err(es) => match es {
                        std::sync::TryLockError::Poisoned(poison_error) => {
                            let mut lock = poison_error.into_inner();
                            if lock.is_none() {
                                Poll::Ready(None)
                            } else {
                                if lock.as_ref().unwrap().0 {
                                    Poll::Ready(None)
                                } else {
                                    let lck = lock.take().unwrap();
                                    Poll::Ready(Some(lck.1))
                                }
                            }
                        }
                        std::sync::TryLockError::WouldBlock => Poll::Pending,
                    },
                }
            }
        }
        Fut { slf: self }
    }

    pub fn respond(&self, msg: Message) -> impl Future<Output = ()> {
        struct Fut<'a> {
            slf: &'a RuntimeHandleInner,
            msg: Option<Message>,
        }
        impl<'a> Future for Fut<'a> {
            type Output = ();

            fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                match self.slf.response.try_lock() {
                    Ok(mut lock) => {
                        *lock = Some((true, self.msg.take().unwrap()));
                        Poll::Ready(())
                    }
                    Err(e) => match e {
                        std::sync::TryLockError::Poisoned(poison_error) => {
                            let mut lock = poison_error.into_inner();
                            *lock = Some((true, self.msg.take().unwrap()));
                            Poll::Ready(())
                        }
                        std::sync::TryLockError::WouldBlock => Poll::Pending,
                    },
                }
            }
        }
        Fut {
            slf: self,
            msg: Some(msg),
        }
    }

    pub fn recieve_response(&self) -> impl Future<Output = Message> {
        struct Fut<'a> {
            slf: &'a RuntimeHandleInner,
        }

        impl<'a> Future for Fut<'a> {
            type Output = Message;
            fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                match self.slf.request.try_lock() {
                    Ok(mut lock) => {
                        if lock.is_none() {
                            Poll::Pending
                        } else if lock.as_ref().unwrap().0 {
                            Poll::Ready(lock.take().unwrap().1)
                        } else {
                            Poll::Pending
                        }
                    }
                    Err(e) => match e {
                        std::sync::TryLockError::Poisoned(poison_error) => {
                            let mut lock = poison_error.into_inner();
                            if lock.is_none() {
                                Poll::Pending
                            } else if lock.as_ref().unwrap().0 {
                                Poll::Ready(lock.take().unwrap().1)
                            } else {
                                Poll::Pending
                            }
                        }
                        std::sync::TryLockError::WouldBlock => Poll::Pending,
                    },
                }
            }
        }
        Fut { slf: self }
    }

    pub fn send_request_immediate(&self, msg: Message) -> impl Future<Output = ()> {
        struct Fut<'a> {
            slf: &'a RuntimeHandleInner,
            msg: Option<Message>,
        }
        impl<'a> Future for Fut<'a> {
            type Output = ();
            fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                match self.slf.request.try_lock() {
                    Ok(mut lock) => {
                        *lock = Some((false, self.msg.take().unwrap()));
                        Poll::Ready(())
                    }
                    Err(e) => match e {
                        std::sync::TryLockError::Poisoned(poison_error) => {
                            let mut lock = poison_error.into_inner();
                            *lock = Some((false, self.msg.take().unwrap()));
                            Poll::Ready(())
                        }
                        std::sync::TryLockError::WouldBlock => Poll::Pending,
                    },
                }
            }
        }
        Fut {
            slf: self,
            msg: Some(msg),
        }
    }

    pub async fn send_request(&self, msg: Message) -> Message {
        self.send_request_immediate(msg).await;
        self.recieve_response().await
    }
}

pub struct RuntimeHandle {
    inner: Arc<RuntimeHandleInner>,
    this_id: ProcessId,
}
impl RuntimeHandle {
    pub fn invalid() -> Self {
        Self {
            inner: Arc::new(RuntimeHandleInner::invalid()),
            this_id: ProcessId::invalid(),
        }
    }
    pub async fn spawn(&self, to_spawn: impl Process + Send + 'static) -> ProcessId {
        let res = self
            .inner
            .send_request(Message::SpawnProcess(Box::new(to_spawn)))
            .await;
        match res {
            Message::ReturnedId(id) => id,
            _ => {
                panic!("unexpected response");
            }
        }
    }
    pub async fn spawn_named(
        &self,
        name: &str,
        to_spawn: impl Process + Send + 'static,
    ) -> ProcessId {
        let res = self
            .inner
            .send_request(Message::SpawnProcessNamed(
                Box::new(to_spawn),
                name.to_string(),
            ))
            .await;
        match res {
            Message::ReturnedId(id) => id,
            _ => {
                panic!("unexpected response");
            }
        }
    }

    pub async fn die(&self) {
        self.inner.send(Message::ProcessExit).await;
    }

    pub async fn kill(&self, id: ProcessId) {
        self.inner.send(Message::KillProcess(id)).await;
    }

    pub async fn check_alive(&self, id: ProcessId) -> bool {
        let mg = self.inner.send_request(Message::DoesProcessExist(id)).await;
        match mg {
            Message::ProcessDoesNotExist => false,
            Message::ReturnedId(_) => true,
            _ => {
                panic!("unexepected message");
            }
        }
    }
    pub async fn get_process(&self, name: &str) -> Option<ProcessId> {
        let res = self
            .inner
            .send_request(Message::GetNamedProcess(name.to_string()))
            .await;
        match res {
            Message::ReturnedId(id) => Some(id),
            Message::ProcessDoesNotExist => None,
            _ => {
                panic!("unexpected response");
            }
        }
    }

    pub async fn send<T: Serialize + DeserializeOwned>(&self, to: ProcessId, value: &T) {
        self.inner
            .send(Message::Message {
                to,
                from: self.this_id,
                data: rmp_serde::to_vec(value).unwrap().into(),
            })
            .await;
    }

    pub async fn recieve<T: Serialize + DeserializeOwned>(
        &self,
    ) -> Result<(ProcessId, T), rmp_serde::decode::Error> {
        let msg = self.inner.receive_wait().await;
        match msg {
            Message::Message { to: _, from, data } => {
                let value: T = rmp_serde::decode::from_slice(&data)?;
                Ok((from, value))
            }
            _ => {
                panic!("unexpected message");
            }
        }
    }

    pub async fn try_recieve<T: Serialize + DeserializeOwned>(&self) -> Option<(ProcessId, T)> {
        let msg = self.inner.receive().await?;
        match msg {
            Message::Message { to: _, from, data } => {
                let Ok(value) = rmp_serde::decode::from_slice(&data) else {
                    return None;
                };
                Some((from, value))
            }
            _ => {
                panic!("unexpected message");
            }
        }
    }

    pub async fn send_request<T: Serialize + DeserializeOwned, U: Serialize + DeserializeOwned>(
        &self,
        to: ProcessId,
        value: &T,
    ) -> Option<U> {
        let msg = self
            .inner
            .send_request(Message::Message {
                to,
                from: self.this_id,
                data: rmp_serde::to_vec(value).unwrap().into(),
            })
            .await;
        match msg {
            Message::Message {
                to: _,
                from: _,
                data,
            } => {
                let Ok(value) = rmp_serde::from_slice(&data) else {
                    return None;
                };
                Some(value)
            }
            Message::ProcessDoesNotExist => None,
            _ => {
                panic!("unexpected message");
            }
        }
    }

    pub async fn get_request<T: Serialize + DeserializeOwned>(&self) -> Option<(ProcessId, T)> {
        let m = self.inner.recieve_request().await?;
        match m {
            Message::Message { to: _, from, data } => {
                let Ok(out) = rmp_serde::from_slice(&data) else {
                    return None;
                };
                Some((from, out))
            }
            _ => {
                panic!("unexpected message type");
            }
        }
    }

    pub async fn respond<T: Serialize + DeserializeOwned>(&self, to: ProcessId, msg: &T) {
        self.inner
            .respond(Message::Message {
                to,
                from: self.this_id,
                data: rmp_serde::to_vec(msg).unwrap().into(),
            })
            .await
    }
}

pub struct Runtime {
    pub processes: Arc<RwLock<BTreeMap<ProcessId, Proc>>>,
    pub proc_names: RwLock<HashMap<String, ProcessId>>,
    pub inflight_requests: RwLock<HashMap<ProcessId, ProcessId>>,
    pub to_run: Mutex<Vec<(ProcessId, ProcessId, Arc<[u8]>)>>,
    pub to_remove: Mutex<Vec<ProcessId>>,
}

impl Runtime {
    pub fn poll(&self) {
        for (id, proc) in &mut self.processes.read().unwrap().iter() {
            let state_res = proc.state.try_lock();
            let mut state = match state_res {
                Ok(s) => s,
                Err(x) => match x {
                    std::sync::TryLockError::Poisoned(poison_error) => poison_error.into_inner(),
                    std::sync::TryLockError::WouldBlock => {
                        continue;
                    }
                },
            };
            let mut ctx = Context::from_waker(Waker::noop());
            let po = state.as_mut().poll(&mut ctx);
            if po.is_ready() {
                self.to_remove.lock().unwrap().push(*id);
            }
        }
    }

    pub fn handle_ip_requests(&self) {
        let mut done = Vec::new();
        let processes = self.processes.read().unwrap();
        let mut requests = self.inflight_requests.write().unwrap();
        for (from, to) in requests.iter() {
            let Some(tor) = processes.get(to) else {
                done.push(*from);
                if let Some(fromr) = processes.get(from) {
                    *fromr.handle.request.lock().unwrap() =
                        Some((true, Message::ProcessDoesNotExist));
                } else {
                }
                continue;
            };
            let mut r = tor.handle.response.lock().unwrap();
            let Some((flag, _)) = r.as_mut() else {
                continue;
            };
            if *flag {
                let rs = r.take().unwrap();
                *processes.get(from).unwrap().handle.request.lock().unwrap() = Some(rs);
                done.push(*from);
            }
        }
        for i in done {
            requests.remove(&i);
        }
    }

    pub fn handle_requests(&self) {
        let mut to_spawn = Vec::new();
        let processes = self.processes.read().unwrap();
        for (pid, proc) in processes.iter() {
            let mut msg = proc.handle.response.lock().unwrap();
            let Some((done, _)) = msg.as_ref() else {
                continue;
            };
            if *done {
                continue;
            }
            let (_, msg2) = msg.take().unwrap();
            let (to, data) = match msg2 {
                Message::Message { to, from: _, data } => (to, data),
                Message::GetNamedProcess(name) => {
                    if let Some(pid) = self.proc_names.read().unwrap().get(&name) {
                        *msg = Some((true, Message::ReturnedId(*pid)));
                    } else {
                        *msg = Some((true, Message::ProcessDoesNotExist));
                    }
                    continue;
                }
                Message::DoesProcessExist(name) => {
                    if let Some(_) = processes.get(&name) {
                        *msg = Some((true, Message::ReturnedId(name)));
                    } else {
                        *msg = Some((true, Message::ProcessDoesNotExist));
                    }
                    continue;
                }
                Message::KillProcess(pid) => {
                    self.to_remove.lock().unwrap().push(pid);
                    continue;
                }
                Message::SpawnProcess(proc) => {
                    to_spawn.push((*pid, proc, None));
                    continue;
                }
                Message::SpawnProcessNamed(proc, name) => {
                    to_spawn.push((*pid, proc, Some(name)));
                    continue;
                }
                _ => {
                    continue;
                }
            };
            self.to_run.lock().unwrap().push((to, *pid, data));
        }
        drop(processes);
        // let processes = self.processes.write().unwrap();
        for (id, proc, name) in to_spawn {
            if !self.processes.read().unwrap().get(&id).is_some() {
                continue;
            }
            let pid = self.spawn_ll(proc);
            let processes = self.processes.read().unwrap();
            *processes.get(&id).unwrap().handle.response.lock().unwrap() =
                Some((true, Message::ReturnedId(pid)));
            if let Some(name) = name {
                self.proc_names.write().unwrap().insert(name, pid);
            }
        }
    }
    pub fn handle_cleanup(&self) {
        let mut processes = self.processes.write().unwrap();
        for (pid, proc) in processes.iter() {
            let mut prc = proc.handle.recieving.lock().unwrap();
            while let Some(msg) = prc.pop_front() {
                match msg {
                    Message::ProcessExit => {
                        self.to_remove.lock().unwrap().push(*pid);
                    }
                    Message::Message { to, from, data } => {
                        let Some(_) = processes.get(&to) else {
                            continue;
                        };
                        if to == *pid {
                            continue;
                        }
                        processes
                            .get(&to)
                            .unwrap()
                            .handle
                            .sending
                            .lock()
                            .unwrap()
                            .push_back(Message::Message { to, from, data });
                    }
                    Message::KillProcess(process_id) => {
                        self.to_remove.lock().unwrap().push(process_id);
                    }
                    Message::GetNamedProcess(_) => todo!(),
                    _ => {}
                }
            }
        }
        let mut to_run_again = Vec::new();
        let mut to_run = self.to_run.lock().unwrap();
        for (to, from, data) in to_run.clone() {
            let Some(proc) = processes.get(&from) else {
                continue;
            };
            let Some(proc2) = processes.get(&to) else {
                *proc.handle.request.lock().unwrap() = Some((true, Message::ProcessDoesNotExist));
                continue;
            };
            {
                if proc2.handle.response.lock().unwrap().is_some() {
                    to_run_again.push((to, from, data));
                    continue;
                }
            }
            *proc2.handle.response.lock().unwrap() = Some((
                true,
                Message::Message {
                    to,
                    from: from,
                    data,
                },
            ));
        }
        *to_run = to_run_again;
        for i in self.to_remove.lock().unwrap().iter() {
            processes.remove(&i);
        }
        *self.to_remove.lock().unwrap() = Vec::new();
    }
    pub fn step(&self) {
        self.poll();
        self.handle_ip_requests();
        self.handle_requests();
        self.handle_cleanup();
    }

    async fn run_process(mut proc: Box<dyn ProcessBase + Send>) {
        loop {
            proc.async_update().await;
            proc_yield().await;
        }
    }

    pub fn spawn(&self, mut proc: impl Process + Send) -> ProcessId {
        let mut min = 1;
        let mut processes = self.processes.write().unwrap();
        for i in 1..=u64::MAX {
            min = i;
            let id = ProcessId { id: i };
            if !processes.contains_key(&id) {
                break;
            }
        }
        if min == u64::MAX {
            panic!("bruh");
        }
        let out = ProcessId { id: min };
        let (h1, h2) = RuntimeHandleInner::new();
        proc.create(
            out,
            RuntimeHandle {
                inner: Arc::new(h1),
                this_id: out,
            },
        );
        let prc = Proc {
            state: Mutex::new(Box::pin(Self::run_process(Box::new(proc)))),
            handle: h2,
        };
        processes.insert(out, prc);
        out
    }

    pub fn spawn_ll(&self, mut proc: Box<dyn ProcessBase + Send>) -> ProcessId {
        let mut processes = self.processes.write().unwrap();
        let mut min = 1;
        for i in 1..=u64::MAX {
            min = i;
            let id = ProcessId { id: i };
            if !processes.contains_key(&id) {
                break;
            }
        }
        if min == u64::MAX {
            panic!("bruh");
        }
        let out = ProcessId { id: min };
        let (h1, h2) = RuntimeHandleInner::new();
        proc.setup(
            out,
            RuntimeHandle {
                inner: Arc::new(h1),
                this_id: out,
            },
        );
        let prc = Proc {
            state: Mutex::new(Box::pin(Self::run_process(proc))),
            handle: h2,
        };
        processes.insert(out, prc);
        out
    }

    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            processes: Arc::new(RwLock::new(BTreeMap::new())),
            inflight_requests: RwLock::new(HashMap::new()),
            to_run: Mutex::new(Vec::new()),
            proc_names: RwLock::new(HashMap::new()),
            to_remove: Mutex::new(Vec::new()),
        })
    }

    pub fn run_loop(self: Arc<Self>) {
        loop {
            let prc = self.processes.read().unwrap();
            if prc.is_empty() {
                break;
            }
            drop(prc);
            self.step();
        }
    }

    pub fn run(self: Arc<Self>, root: impl Process + Send + 'static) {
        self.spawn(root);
        let mut handles = Vec::new();
        let tc = std::thread::available_parallelism().unwrap().get() - 1;
        for _ in 0..tc {
            let ac = self.clone();
            handles.push(std::thread::spawn(move || {
                ac.run_loop();
            }));
        }
        self.run_loop();
        for i in handles {
            i.join().unwrap();
        }
    }
}

pub fn proc_yield() -> impl Future<Output = ()> {
    pub struct Out {
        woken: bool,
    }
    impl Future for Out {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.woken {
                Poll::Ready(())
            } else {
                self.woken = true;
                Poll::Pending
            }
        }
    }
    Out { woken: false }
}

pub struct AsyncStream {
    inner: TcpStream,
    lock: Mutex<()>,
}

impl AsyncStream {
    pub fn read_buf_no_prefix<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<(), std::io::Error>> {
        pub struct Out<'a> {
            slf: &'a mut AsyncStream,
            buf: Option<&'a mut [u8]>,
        }
        impl<'a> Future for Out<'a> {
            type Output = Result<(), std::io::Error>;

            fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                let buf = self.buf.take().unwrap();
                let oupt = self.slf.inner.read_exact(buf);
                match oupt {
                    Ok(_) => {
                        return Poll::Ready(Ok(()));
                    }
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::WouldBlock => {
                            self.buf = Some(buf);
                            Poll::Pending
                        }
                        _ => Poll::Ready(Err(e)),
                    },
                }
            }
        }
        Out {
            slf: self,
            buf: Some(buf),
        }
    }
    pub fn write_buf_no_prefix<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<(), std::io::Error>> {
        pub struct Out<'a> {
            slf: &'a mut AsyncStream,
            buf: Option<&'a [u8]>,
            count: usize,
        }
        impl<'a> Future for Out<'a> {
            type Output = Result<(), std::io::Error>;
            fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
                let buf = self.buf.take().unwrap();
                let count = self.count;
                let oupt = self.slf.inner.write(&buf[count..]);
                match oupt {
                    Ok(size) => {
                        if size + self.count == buf.len() {
                            return Poll::Ready(Ok(()));
                        } else {
                            self.count += size;
                            return Poll::Pending;
                        }
                    }
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::WouldBlock => {
                            self.buf = Some(buf);
                            Poll::Pending
                        }
                        _ => Poll::Ready(Err(e)),
                    },
                }
            }
        }
        Out {
            slf: self,
            buf: Some(buf),
            count: 0,
        }
    }

    pub async fn read_u64(&mut self) -> Result<u64, std::io::Error> {
        let mut buf = [0; 8];
        self.read_buf_no_prefix(&mut buf).await?;
        let out = u64::from_le_bytes(buf);
        Ok(out)
    }

    pub async fn write_u64(&mut self, value: u64) -> Result<(), std::io::Error> {
        let buf = u64::to_le_bytes(value);
        self.write_buf_no_prefix(&buf).await?;
        return Ok(());
    }

    pub async fn write_buffer(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        self.write_u64(buf.len() as u64).await?;
        self.inner.set_nonblocking(false)?;
        let out = self.inner.write(buf);
        self.inner.set_nonblocking(true)?;
        if let Err(e) = out { Err(e) } else { Ok(()) }
    }

    pub async fn read_buffer(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let count = self.read_u64().await?;
        self.inner.set_nonblocking(false)?;
        let mut buf = vec![0; count as usize];
        let out = self.inner.read_exact(&mut buf);
        self.inner.set_nonblocking(true)?;
        if let Err(e) = out { Err(e) } else { Ok(buf) }
    }

    pub async fn write_object<T: Serialize + DeserializeOwned>(
        &mut self,
        value: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = rmp_serde::to_vec(&value)?;
        Ok(self.write_buffer(&bytes).await?)
    }

    pub async fn read_object<T: Serialize + DeserializeOwned>(
        &mut self,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let bytes = self.read_buffer().await?;
        let out = rmp_serde::from_slice(&bytes)?;
        Ok(out)
    }
}
