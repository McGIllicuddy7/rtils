use rtils::{ContextMut, ContextRef, context};

pub fn main() { 
    context().add("i32", 0);
    let mut a:ContextMut<i32> = context().get_mut("i32").unwrap();
    *a+=1;
    drop(a);
    let b:ContextRef<i32> = context().get("i32").unwrap();
    println!("{:#?}", *b);
}
