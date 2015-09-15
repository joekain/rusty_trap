use libc::pid_t;

#[derive(Copy, Clone)]
pub enum InferiorState {
    Running,
    Stopped,
    SingleStepping
}

#[derive(Copy, Clone)]
pub struct Inferior {
    pub pid: pid_t,
    pub state: InferiorState
}

pub type TrapInferior = pid_t;
