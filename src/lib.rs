use std::{cell::UnsafeCell, ops::{Deref, DerefMut}};
pub use std::{any::Any, collections::HashMap};
pub trait ContextItem: Any + Send +'static +{
    fn duplicate(&self) -> Box<dyn ContextItem>;
}
impl<T: Clone + Any + Send> ContextItem for T {
    fn duplicate(&self) -> Box<dyn ContextItem> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn ContextItem> {
    fn clone(&self) -> Self {
        self.duplicate()
    }
}
struct ItemRef{
    mutably_borrowed:bool, 
    borrow_count:usize,
    item:Box<dyn ContextItem>,
}
#[derive(Debug)]
pub struct ContextRef<T:ContextItem>{
    pub item:&'static T,
    pub key:String
}
#[derive(Debug)]
pub struct ContextMut<T:ContextItem>{
    pub item:&'static mut T,
    pub key:String
}

pub struct Context { 
    items:UnsafeCell<HashMap<String, Box<ItemRef>>>,
}
thread_local!(
    static LOCAL_CONTEXT:UnsafeCell<Context> = UnsafeCell::new(Context::new());
);
pub fn context()->&'static Context{
    unsafe{
        LOCAL_CONTEXT.with(
            |a|{
                a.get().as_ref().expect("always a valid pointer")
            }
        )
    }
}
impl<T:ContextItem> Drop for ContextRef<T>{
    fn drop(&mut self) {
        unsafe{
            context().drop_ref(self.key.clone())
        }
    }
}
impl <T:ContextItem> Deref for ContextRef<T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.item
    }
}
impl <T:ContextItem> Drop for ContextMut<T>{
    fn drop(&mut self){
        unsafe{
            context().drop_mut(self.key.clone())
        }
    }
}
impl <T:ContextItem> Deref for ContextMut<T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.item
    }
}
impl <T:ContextItem> DerefMut for ContextMut<T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item
    }
}
impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context{
    pub fn new()->Self{
        Self { items: UnsafeCell::new(HashMap::new()) }
    } 
    unsafe fn drop_ref(&self, name:String){
        unsafe{
            if let Some(st) = self.items.get().as_mut().unwrap().get_mut(&name){
                st.borrow_count -=1;
            }
        }
    } 
    unsafe fn drop_mut(&self, name:String){
        unsafe{
            if let Some(st) = self.items.get().as_mut().unwrap().get_mut(&name){
                st.mutably_borrowed = false;
            }
        }
    }
    pub fn add<T:ContextItem, U:Into<String>>(&self, name:U, item:T){
        unsafe{
            let it = Box::new(ItemRef{mutably_borrowed:false, borrow_count:0,item: Box::new(item)});
            self.items.get().as_mut().unwrap().insert(name.into(), it);
        }
    }
    pub fn get<T:ContextItem>(&self, name:&str)->Result<ContextRef<T>,()>{
        unsafe{
            let Some(t) = self.items.get().as_mut().unwrap().get_mut(name)else {
                return Err(());
            };
            if t.mutably_borrowed{
                return Err(());
            }
            t.borrow_count += 1;
            let it:&dyn Any= t.item.as_ref();
            let Some(item) = it.downcast_ref()else {
                return Err(());
            };
            let out = ContextRef{key:name.to_string(), item};
            Ok(out)
        }
    }
    pub fn get_mut<T:ContextItem>(&self, name:&str)->Result<ContextMut<T>,()>{
        unsafe{
            let Some(t) = self.items.get().as_mut().unwrap().get_mut(name)else {
                return Err(());
            };
            if t.mutably_borrowed{
                return Err(());
            }
            if t.borrow_count != 0{
                return Err(());
            }
            t.mutably_borrowed = true;
            let it:&mut dyn Any= t.item.as_mut();
            let Some(item) = it.downcast_mut()else {
                return Err(());
            };
            let out = ContextMut{key:name.to_string(), item};
            Ok(out)
        }
    }
    pub fn remove(&self, key:&str)->Result<(),()>{
        unsafe{
            let items = self.items.get().as_mut().unwrap();
            if !items.contains_key(key){
                return Err(());
            }
            let item = items.get_mut(key).unwrap();
            if item.mutably_borrowed || item.borrow_count != 0{
                return Err(());
            }
            items.remove(key);
            Ok(())
        }
    }
}

#[test]
fn basics(){
    context().add("i32", 0);
    let mut a:ContextMut<i32> = context().get_mut("i32").unwrap();
    *a+=1;
    drop(a);
    let b:ContextRef<i32> = context().get("i32").unwrap();
    assert!(*b == 1);
    drop(b);
    context().remove("i32").unwrap();
}
#[test]
fn threads(){
    context().add("i32", 0);
    let handle = std::thread::spawn(||{
        _ = context().get::<i32>("i32").expect_err("should not exist");
    });
    handle.join().unwrap();
}
#[test]
fn borrowing(){
    context().add("i32", 0);
    {
    
        let _a:ContextRef<i32> = context().get("i32").unwrap(); 
        let _b:ContextRef<i32> = context().get("i32").unwrap();
        let _ = context().get_mut::<i32>("i32").expect_err("should not be allowed to read");  
    }
    {
        let _x= context().get_mut::<i32>("i32").unwrap();
        let _ = context().get_mut::<i32>("i32").unwrap();
    }
}