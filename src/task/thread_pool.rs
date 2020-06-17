use std::sync::{Mutex, Arc};
use crate::task::Task;
use std::sync::mpsc::{Sender, Receiver, channel, SendError};
use std::sync::atomic;
use std::sync::atomic::{AtomicUsize};
use std::thread::Builder;

struct ThreadPoolHeader {
    name: Option<String>,
    stack_size: Option<usize>,
    thread_nums: AtomicUsize,
    task_wait_nums: AtomicUsize,
    task_running_nums: AtomicUsize,
    panic_thread_nums: AtomicUsize,
    task_recv: Mutex<Receiver<Box<dyn Task + Send>>>,
}

pub struct ThreadPool {
    header: Arc<ThreadPoolHeader>,
    task_send: Sender<Box<dyn Task + Send>>,
}

pub struct ThreadPoolBuilder {
    name: Option<String>,
    stack_size: Option<usize>,
    thread_nums: Option<usize>,
}

struct PoolGuard {
    header: Arc<ThreadPoolHeader>,
    is_active: bool,
}

impl PoolGuard {
    fn new(header: Arc<ThreadPoolHeader>) -> PoolGuard {
        header.thread_nums.fetch_add(1, atomic::Ordering::SeqCst);
        PoolGuard {
            header: header,
            is_active: true,
        }
    }
    
    fn normal_exit(&mut self) {
        self.is_active = false;
    }
}

impl Drop for PoolGuard {
    fn drop(&mut self) {
        if self.is_active {
            self.header.panic_thread_nums.fetch_sub(1, atomic::Ordering::SeqCst);
        }
        self.header.thread_nums.fetch_sub(1, atomic::Ordering::SeqCst);
    }
}

impl ThreadPoolBuilder {
    pub fn new() -> ThreadPoolBuilder {
        ThreadPoolBuilder {
            name: None,
            stack_size: None,
            thread_nums: None,
        }
    }
    
    pub fn pool_name(mut self, name: String) -> ThreadPoolBuilder {
        self.name = Some(name);
        self
    }
    
    pub fn stack_size(mut self, size_: usize) -> ThreadPoolBuilder {
        self.stack_size = Some(size_);
        self
    }
    
    pub fn thread_numbers(mut self, nums: usize) -> ThreadPoolBuilder {
        self.thread_nums = Some(nums);
        self
    }
    
    pub fn spawn(self) -> ThreadPool {
        let (tx, rx) = channel();
        
        let header = Arc::new(ThreadPoolHeader {
            name: self.name,
            stack_size: self.stack_size,
            thread_nums: AtomicUsize::new(0),
            task_wait_nums: AtomicUsize::new(0),
            task_running_nums: AtomicUsize::new(0),
            panic_thread_nums: AtomicUsize::new(0),
            task_recv: Mutex::new(rx),
        });
        
        for i in 0..self.thread_nums.unwrap() {
            let name = format!("{}", i);
            ThreadPoolBuilder::spawn_thread(header.clone(), &name);
        }
        
        ThreadPool {
            header,
            task_send: tx.clone(),
        }
    }
    
    fn spawn_thread(header: Arc<ThreadPoolHeader>, name: &String) {
        let mut builder = Builder::new();
        if let Some(s) = header.stack_size {
            builder = builder.stack_size(s);
        }
        let n = if let Some(n) = &header.name {
            let mut tmp = String::from(n);
            tmp.push(':');
            tmp.push_str(name.as_str());
            tmp
        } else {
            let mut tmp = String::from(":");
            tmp.push_str(name.as_str());
            tmp
        };
        
        builder.name(n.clone()).spawn(move || {
            let mut guard = PoolGuard::new(header.clone());
            loop {
                let task = {
                   let lock = header.task_recv.lock().expect("The thread {} unable to receive task");
                    lock.recv()
                };
                
                match task {
                    Ok(task) => {
                        header.task_running_nums.fetch_add(1, atomic::Ordering::SeqCst);
                        header.task_wait_nums.fetch_sub(1, atomic::Ordering::SeqCst);
                        
                        task.run();
                        
                        header.task_running_nums.fetch_sub(1, atomic::Ordering::SeqCst);
                    },
                    Err(_) => break,
                }
            }
            
            guard.normal_exit();
        }).unwrap();
    }
}

impl ThreadPool {
    pub fn thread_num(&self) -> usize {
        self.header.thread_nums.load(atomic::Ordering::Acquire)
    }
    
    pub fn pool_name(&self) -> Option<String> {
        self.header.name.clone()
    }
    
    pub fn task_running_num(&self) -> usize {
        self.header.task_running_nums.load(atomic::Ordering::Acquire)
    }
    
    pub fn task_wait_num(&self) -> usize {
        self.header.task_wait_nums.load(atomic::Ordering::Acquire)
    }
    
    pub fn panic_thread_num(&self) -> usize {
        self.header.panic_thread_nums.load(atomic::Ordering::Acquire)
    }
    
    pub fn execute<T: Task + Send + 'static>(&self, task: T) -> Result<(), SendError<Box<dyn Task + Send>>> {
        let r = self.task_send.send(Box::new(task));
        if r.is_ok() {
            self.header.task_wait_nums.fetch_add(1, atomic::Ordering::SeqCst);
        }
        r
    }
}
