use breakpoint::Breakpoint;
use libc::c_void;
use libc::pid_t;
use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, Sub};
use object;
use std::fs;
use std::path::Path;


pub struct TrapData<'a> {
    pub filename: &'a Path,
    pub data: Vec<u8>,
}

impl <'a> TrapData<'a> {
    pub fn new(filename: &Path) -> TrapData {
	TrapData {
	    filename: filename,
	    data: fs::read(filename).unwrap()
	}
    }
}

#[derive(Copy, Clone)]
pub enum InferiorState {
    Running,
    Stopped,
    SingleStepping,
}

pub struct TrapInferior<'a> {
    pub pid: pid_t,
    pub state: InferiorState,
    pub breakpoints: HashMap<InferiorPointer, Breakpoint>,
    obj: object::File<'a>,
}

impl <'a> TrapInferior<'a> {
    pub fn new(pid: pid_t, trap_data: &'a TrapData<'a>) -> TrapInferior<'a> {
        TrapInferior {
            pid,
            state: InferiorState::Stopped,
            breakpoints: HashMap::new(),
            obj: object::File::parse(&*trap_data.data).unwrap(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Eq, Hash)]
pub struct InferiorPointer(pub u64);
impl InferiorPointer {
    pub fn as_voidptr(&self) -> *mut c_void {
        let &InferiorPointer(u) = self;
        u as *mut c_void
    }

    pub fn as_i64(&self) -> i64 {
        let &InferiorPointer(u) = self;
        u as i64
    }
}
impl Add<i64> for InferiorPointer {
    type Output = InferiorPointer;
    fn add(self, rhs: i64) -> InferiorPointer {
        let InferiorPointer(u) = self;
        if rhs >= 0 {
            InferiorPointer(u + rhs as u64)
        } else {
            InferiorPointer(u - rhs as u64)
        }
    }
}
impl Sub<i64> for InferiorPointer {
    type Output = InferiorPointer;
    fn sub(self, rhs: i64) -> InferiorPointer {
        let InferiorPointer(u) = self;
        if rhs >= 0 {
            InferiorPointer(u - rhs as u64)
        } else {
            InferiorPointer(u + rhs as u64)
        }
    }
}
impl fmt::Display for InferiorPointer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &InferiorPointer(u) = self;
        write!(f, "{}", u)
    }
}
