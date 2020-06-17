/// 状态机  

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::cell::{RefCell, Cell};
use std::thread::ThreadId;
use crate::task::task::Task;
use std::rc::Rc;

pub trait State {
    /// 状态机id, 状态之间需保证id的唯一性  
    fn state_id(&self) -> u64;
    
    /// 状态回调函数  
    fn call(&self);
    
    /// 当前状态的下一个状态  
    fn next_state(&self) -> Option<u64>;
}

/// 状态机当前只支持在一个线程中执行状态, 但支持在多个线程中更新状态;  
/// 当状态机执行时, 状态机的状态为空, 或者从一个状态跳转到空状态时, 则会退出状态机执行;  
/// 在状态切换到空之前, self.run()一直会阻塞当前线程;  
/// 
/// # panics
/// 
/// 当状态机已经在一个线程中执行, 在另一个线程中run时, 会产生panic, 可能会引起执行该状态机的线程panic;  
#[derive(Clone)]
pub struct StateMachine {
    env_id: Cell<Option<ThreadId>>,
    env_name: Rc<Cell<String>>,
    cur: Cell<Option<u64>>,
    is_run: Arc<Mutex<bool>>,
    map: Arc<RwLock<RefCell<HashMap<u64, Box<dyn State>>>>>,
}

impl StateMachine {
    pub fn new(init_sta: Option<Box<dyn State>>) -> StateMachine {
        let mut h = HashMap::new();
        let cur = match init_sta {
            None => None,
            Some(sta) => {
                let id = sta.state_id();
                h.insert(id, sta);
                Some(id)
            },
        };
        
        StateMachine {
            env_id: Cell::new(None),
            env_name: Rc::new(Cell::new(String::new())),
            cur: Cell::new(cur),
            map: Arc::new(RwLock::new(RefCell::new(h))),
            is_run: Arc::new(Mutex::new(false)),
        }
    }
}

impl StateMachine
{
    /// 向状态机插入状态, 
    pub fn insert(&self, sta: Box<dyn State>) {
        self.map.write().unwrap().borrow_mut().insert(sta.state_id(), sta);
    }
}

impl Task for StateMachine {
    fn run(&self) {
        let err = {
            let cur_thread = std::thread::current();
            let mut lock = self.is_run.lock().unwrap();
            let env_name = self.env_name.take();
            if *lock {
                self.env_name.set(env_name.clone());
                format!("Cannot run in the `Thread(id:{:?}, name: {})`, already running in the `Thread(id: {:?}, name: {})`",
                                  cur_thread.id(), cur_thread.name().unwrap_or(""), self.env_id.get().unwrap(), env_name)
            } else {
                *lock = true;
                self.env_id.replace(Some(cur_thread.id()));
                self.env_name.replace(match cur_thread.name() { Some(x) => String::from(x), None => String::from("")});
                String::new()
            }
        };
        
        if !err.is_empty() {
            panic!(err);
        }
        
        loop {
            match self.cur.get() {
                None => break,
                Some(id) => {
                    let lock = self.map.read().unwrap();
                    let h = lock.borrow();
                    let evt = h.get(&id).unwrap();
                    
                    evt.call();
                    
                    self.cur.set(evt.next_state());
                },
            }
        } // loop
        
        *self.is_run.lock().unwrap() = false;
    } // run
}

unsafe impl std::marker::Send for StateMachine {}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicI32;
    use std::sync::Arc;

    struct Threshold {
        high: i32,
        low: i32,
    }
    
    #[repr(C)]
    enum StateId {
        Normal,
        ProUnderCellVol,
        ProOverCellVol,
        ProUnderCellTemp,
        ProOverCellTemp,
        ProUnderBatCur,
        ProOverBatCur,
        ProUnderBatVol,
        ProOverBatVol,
    }
    
    const CELL_NUM_OF_BATTERY: usize = 15;
    
    struct BatInfo {
        /// vol * 1000
        cellVol: [AtomicI32; CELL_NUM_OF_BATTERY],
        /// temp * 100
        cellTemp: [AtomicI32; CELL_NUM_OF_BATTERY],
        /// cur * 1000
        batCur: [AtomicI32; CELL_NUM_OF_BATTERY],
        /// vol * 1000
        batVol: AtomicI32,
        /// temp * 100
        batTemp: AtomicI32,
    }
    
    struct ProThreshold {
        cellVol: Threshold,
        batCur: Threshold,
        batVol: Threshold,
        cellTemp: Threshold,
        batTemp: Threshold,
    }
    
    struct BatNormal {
        id: StateId,
        bat_info: Arc<BatInfo>,
        pro_para: Arc<ProThreshold>,
    }
    
    struct CellUnderVoltage {
        id: StateId,
        bat_info: Arc<BatInfo>,
        pro_para: Arc<ProThreshold>,
    }
    //...
}
