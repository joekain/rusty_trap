use libc::c_void;
use libc::pid_t;
use std::ops::{Add, Sub};

#[derive(Copy, Clone)]
pub enum InferiorState {
    Running,
    Stopped,
    SingleStepping,
}

#[derive(Copy, Clone)]
pub struct Inferior {
    pub pid: pid_t,
    pub state: InferiorState,
}

pub type TrapInferior = pid_t;

#[derive(Copy, Clone)]
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
