use erl::erl::{Process, ProcessId, Runtime, RuntimeHandle};
pub struct Test {
    pub handle: RuntimeHandle,
    pub this_id: ProcessId,
    pub primes: Vec<usize>,
    pub counter: usize,
}
impl Process for Test {
    fn create(&mut self, this_id: ProcessId, handle: RuntimeHandle) {
        self.handle = handle;
        self.this_id = this_id;
    }

    async fn update(&mut self) {
        if self.counter < 2 {
            self.counter = 2;
        }
        self.counter += 1;
        let mut is_prime = true;
        for i in &self.primes {
            if self.counter % *i == 0 {
                is_prime = false;
                break;
            }
        }
        if is_prime {
            self.primes.push(self.counter);
        }
        if self.counter > 1000000 {
            // println!("died at:{:#?}", self.counter);
            self.handle.die().await;
        }
        while let Some((i, st)) = self.handle.try_recieve::<String>().await {
            println!("i:{} said:{}", i.get(), st);
        }
    }
}

pub struct Root {
    pub handle: RuntimeHandle,
    pub this_id: ProcessId,
    pub children: Vec<ProcessId>,
    pub counter: usize,
}
impl Process for Root {
    fn create(&mut self, this_id: ProcessId, handle: RuntimeHandle) {
        self.handle = handle;
        self.this_id = this_id;
    }
    async fn update(&mut self) {
        if self.children.is_empty() {
            for _ in 0..1000 {
                let pid = self
                    .handle
                    .spawn(Test {
                        primes: Vec::new(),
                        handle: RuntimeHandle::invalid(),
                        this_id: ProcessId::invalid(),
                        counter: 0,
                    })
                    .await;
                self.children.push(pid);
            }
        } else {
            let mut dead = Vec::new();
            for i in &self.children {
                if !self.handle.check_alive(*i).await {
                    dead.push(*i);
                }
                self.counter += 1;
            }
            for i in dead {
                for j in 0..self.children.len() {
                    if self.children[j] == i {
                        self.children.remove(j);
                        break;
                    }
                }
            }
            if self.children.is_empty() {
                //   println!("died with:{}", self.counter);
                self.handle.die().await;
            }
        }
    }
}
pub fn main() {
    let rt = Runtime::new();
    rt.run(Root {
        counter: 0,
        handle: RuntimeHandle::invalid(),
        this_id: ProcessId::invalid(),
        children: Vec::new(),
    });
}
