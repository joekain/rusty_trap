extern crate libc;
extern crate nix;
extern crate object;

use libc::pid_t;
use nix::sys::wait::*;
use nix::unistd::Pid;
use nix::{
    sys::signal,
    sys::wait::waitpid,
    unistd::{execve, fork, ForkResult},
    Error,
};
use std::ffi::{CStr, CString};
use std::path::Path;

mod ptrace_util;

mod inferior;
use inferior::*;

mod breakpoint;

pub use self::breakpoint::trap_inferior_set_breakpoint;
use breakpoint::TrapBreakpoint;

fn disable_address_space_layout_randomization() {
    unsafe {
        let old = libc::personality(0xffffffff);
        libc::personality((old | libc::ADDR_NO_RANDOMIZE) as u64);
    }
}

fn exec_inferior(filename: &Path, _args: &[&str]) {
    // let c_filename = &CStr::from_ptr(filename.to_str().unwrap().as_ptr() as *const i8);
    let cstring_filename = CString::new(
        filename
            .to_str()
            .expect("Failed to get string from filename"),
    )
    .expect("Failed to create CString from filename");
    disable_address_space_layout_randomization();
    ptrace_util::trace_me();
    let cstr_filename = unsafe { CStr::from_ptr(cstring_filename.as_ptr()) };
    execve::<CString, CString>(cstr_filename, &[], &[]).expect("Failed execve");
    unreachable!();
}

fn attach_inferior<'a>(raw_pid: pid_t, data: &'a TrapData) -> Result<TrapInferior<'a>, Error> {
    let nix_pid = Pid::from_raw(raw_pid);
    match waitpid(nix_pid, None) {
        Ok(WaitStatus::Stopped(pid, signal::Signal::SIGTRAP)) => Ok(TrapInferior::new(pid.into(), data)),
        Ok(_) => panic!("Unexpected stop in attach_inferior"),
        Err(e) => Err(e),
    }
}

pub fn trap_inferior_exec<'a>(data: &'a TrapData, args: &[&str]) -> Result<TrapInferior<'a>, Error> {
    loop {
        match unsafe { fork() } {
            Ok(ForkResult::Child) => {
                exec_inferior(data.filename, args);
                unreachable!();
            }
            Ok(ForkResult::Parent { child: pid }) => return attach_inferior(pid.into(), data),
            Err(Error::EAGAIN) => continue,
            Err(e) => return Err(e),
        }
    }
}

pub fn trap_inferior_continue<F>(mut inferior: TrapInferior, mut callback: F) -> (TrapInferior, i32)
where
    F: FnMut(&TrapInferior, TrapBreakpoint),
{
    inferior.state = InferiorState::Running;
    ptrace_util::cont(inferior.pid);
    loop {
        inferior.state = match waitpid(Pid::from_raw(inferior.pid), None) {
            Ok(WaitStatus::Exited(_pid, code)) => return (inferior, code),
            Ok(WaitStatus::Stopped(_pid, signal::SIGTRAP)) => {
                breakpoint::handle(&mut inferior, &mut callback)
            }
            Ok(WaitStatus::Stopped(_pid, signal)) => {
                panic!(
                    "Unexpected stop on signal {} in trap_inferior_continue.  State: {}",
                    signal, inferior.state as i32
                )
            }
            Ok(_) => panic!("Unexpected stop in trap_inferior_continue"),
            Err(_) => panic!("Unhandled error in trap_inferior_continue"),
        };
    }
}
