extern crate nix;
extern crate libc;

use nix::unistd::*;
use nix::unistd::Fork::*;
use libc::pid_t;
use nix::Error;
use nix::errno;
use nix::sys::wait::*;
use std::ffi::CString;
use std::path::Path;
use nix::sys::signal;

mod ptrace_util;
use ptrace_util::inferior_pointer::InferiorPointer;

mod inferior;
use inferior::*;

mod breakpoint;

pub use self::breakpoint::trap_inferior_set_breakpoint;
use breakpoint::{handle_breakpoint, TrapBreakpoint};

#[derive(Copy, Clone)]
struct Breakpoint {
    shift : u64,
    target_address  : InferiorPointer,
    aligned_address : InferiorPointer,
    original_breakpoint_word : i64
}

static mut global_breakpoint : Breakpoint = Breakpoint {
    shift: 0,
    target_address: InferiorPointer(0),
    aligned_address: InferiorPointer(0),
    original_breakpoint_word: 0
};
static mut global_inferior : Inferior = Inferior { pid: 0, state: InferiorState::Stopped };

mod ffi {
    use libc::{c_int, c_long};

    extern {
        pub fn personality(persona: c_long) -> c_int;
    }
}

fn disable_address_space_layout_randomization() -> () {
    unsafe {
        let old = ffi::personality(0xffffffff);
        ffi::personality((old | 0x0040000) as i64);
    }
}

fn exec_inferior(filename: &Path, args: &[&str]) -> () {
    let c_filename = &CString::new(filename.to_str().unwrap()).unwrap();
    disable_address_space_layout_randomization();
    ptrace_util::trace_me();
    execve(c_filename, &[], &[])
        .ok()
        .expect("Failed execve");
    unreachable!();
}

fn attach_inferior(pid: pid_t) -> Result<Inferior, Error> {
    match waitpid(pid, None) {
        Ok(WaitStatus::Stopped(pid, signal::SIGTRAP)) =>
            return Ok(Inferior {pid: pid, state: InferiorState::Running}),
        Ok(_) => panic!("Unexpected stop in attach_inferior"),
        Err(e) => return Err(e)
    }
}

pub fn trap_inferior_exec(filename: &Path, args: &[&str]) -> Result<TrapInferior, Error> {
    loop {
        match fork() {
            Ok(Child)                      => exec_inferior(filename, args),
            Ok(Parent(pid))                => {
                unsafe { global_inferior = attach_inferior(pid).ok().unwrap() };
                return Ok(pid)
            },
            Err(Error::Sys(errno::EAGAIN)) => continue,
            Err(e)                         => return Err(e)
        }
    }
}

pub fn trap_inferior_continue<F>(inferior: TrapInferior, callback: &mut F) -> i8
    where F: FnMut(TrapInferior, TrapBreakpoint) -> () {

    let mut inf = unsafe { global_inferior };
    ptrace_util::cont(inf.pid);
    loop {
        inf.state = match waitpid(inf.pid, None) {
            Ok(WaitStatus::Exited(_pid, code)) => return code,
            Ok(WaitStatus::Stopped(_pid, signal::SIGTRAP)) =>
                handle_breakpoint(inf, callback),
            Ok(WaitStatus::Stopped(_pid, signal)) => {
                panic!("Unexpected stop on signal {} in trap_inferior_continue.  State: {}", signal, inf.state as i32)
            },
            Ok(_) => panic!("Unexpected stop in trap_inferior_continue"),
            Err(_) => panic!("Unhandled error in trap_inferior_continue")
        };

        unsafe { global_inferior = inf };
    }
}
