pub mod allocator;
pub mod device;
pub mod input;
pub mod io_t;
pub mod rtils;

use std::{
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard},
};

#[allow(unused)]
use crate::input::Input;
pub use allocator::{
    CharPtr, Ptr, SharedPtr, calloc, calloc_shared, malloc, malloc_shared, realloc, realloc_shared,
};
pub use io_t::*;
pub use raylib::{ffi::KeyboardKey, math::Vector2};
pub use rtils::marathon::BStream;
#[allow(unused)]
pub use rtils::rtils_useful::{Global, GlobalConst};
#[macro_export]
macro_rules! _start {
    ($contents:block) => {
        pub fn main() {
            let cmd = amiga::setup();
            fn inner_main() $contents;

            if let Some(c) = cmd{
                std::thread::spawn(inner_main);
                amiga::main_loop(c);
            }else{
                inner_main();
            }

        }
    };
}

pub use device::main_loop;
use serde::{Deserialize, Serialize};

static HANDLE: Mutex<Option<Handle>> = Mutex::new(None);
struct Handle {
    input: Input,
    queue: BStream<Cmd>,
    pressed_key: Option<char>,
    should_close: bool,
    updated: bool,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct HandleUpdate {
    input: Input,
    pressed_key: Option<char>,
    should_close: bool,
}
struct CmdHandle {
    lock: MutexGuard<'static, Option<Handle>>,
}

impl Deref for CmdHandle {
    type Target = Handle;
    fn deref(&self) -> &Self::Target {
        self.lock.as_ref().unwrap()
    }
}
impl DerefMut for CmdHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.lock.as_mut().unwrap()
    }
}
fn get_handle() -> CmdHandle {
    CmdHandle {
        lock: HANDLE.lock().unwrap(),
    }
}
