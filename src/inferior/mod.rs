use libc::pid_t;
use libc::c_void;
use std::ops::{Add, Sub};
use std::any::Any;
use std::vec::Vec;

#[derive(Copy, Clone)]
pub enum InferiorState {
    Running,
    Stopped,
    SingleStepping
}

//#[derive(Copy, Clone)]
pub struct Inferior {
    pub pid: pid_t,
    pub state: InferiorState,
    pub privates: Vec<Box<Any>>
}

pub type TrapInferior = Box<Inferior>;

#[derive(Copy, Clone)]
pub struct InferiorPointer(pub u64);
impl InferiorPointer {
    pub fn as_voidptr(&self) -> * mut c_void {
        let &InferiorPointer(u) = self;
        u as * mut c_void
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
