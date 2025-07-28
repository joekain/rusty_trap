use breakpoint::Breakpoint;
use libc::c_void;
use libc::pid_t;
use object;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader};
use std::ops::{Add, Sub};
use std::path::Path;

pub struct TrapData<'a> {
    pub filename: &'a Path,
    pub data: Vec<u8>,
}

impl<'a> TrapData<'a> {
    pub fn new(filename: &Path) -> TrapData {
        TrapData {
            filename,
            data: fs::read(filename).unwrap(),
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
    pub obj: object::File<'a>,
    pub base_address: u64,
}

impl<'a> TrapInferior<'a> {
    pub fn new(pid: pid_t, trap_data: &'a TrapData<'a>) -> TrapInferior<'a> {
        TrapInferior {
            pid,
            state: InferiorState::Stopped,
            breakpoints: HashMap::new(),
            obj: object::File::parse(&*trap_data.data).unwrap(),
            base_address: get_base_address(pid, trap_data.filename),
        }
    }
}

// Helper function to look up the base address by inspected /proc/pid/maps
fn get_base_address(pid: pid_t, filename: &Path) -> u64 {
    let proc_filename = format!("/proc/{pid}/maps");
    let file = fs::File::open(proc_filename).expect("Unable to open file");
    let expected = fs::canonicalize(filename).unwrap();
    let expected = expected.to_str().unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        if line.contains(expected) {
            let addr_str = line.split('-').next().unwrap();
            println!("Found base address 0x{addr_str}");
            return u64::from_str_radix(addr_str, 16).unwrap();
        }
    }
    // This should be an error, there should be error handling.
    println!("Could not find base address for {expected}");
    panic!();
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
        write!(f, "{u}")
    }
}
