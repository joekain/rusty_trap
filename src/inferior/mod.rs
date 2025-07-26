use breakpoint::Breakpoint;
use libc::c_void;
use libc::pid_t;
use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, Sub};

#[derive(Copy, Clone)]
pub enum InferiorState {
    Running,
    Stopped,
    SingleStepping,
}

#[derive(Clone)]
pub struct TrapInferior {
    pub pid: pid_t,
    pub state: InferiorState,
    pub breakpoints: HashMap<InferiorPointer, Breakpoint>,
}

impl TrapInferior {
    pub fn new(pid: pid_t) -> TrapInferior {
        TrapInferior {
            pid,
            state: InferiorState::Stopped,
            breakpoints: HashMap::new(),
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
