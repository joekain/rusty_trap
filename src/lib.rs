extern crate nix;
extern crate libc;

use nix::unistd::*;
use nix::unistd::Fork::*;
use libc::pid_t;
use nix::Error;
use nix::errno;
use nix::sys::ptrace::*;
use nix::sys::ptrace::ptrace::*;
use nix::sys::wait::*;
use std::ffi::CString;
use std::ptr;
use std::path::Path;
use nix::sys::signal;


pub type TrapInferior = pid_t;

fn exec_inferior(filename: &Path, args: &[&str]) -> () {
    let c_filename = &CString::new(filename.to_str().unwrap()).unwrap();
    ptrace(PTRACE_TRACEME, 0, ptr::null_mut(), ptr::null_mut())
        .ok()
        .expect("Failed PTRACE_TRACEME");
    execve(c_filename, &[], &[])
        .ok()
        .expect("Failed execve");
    unreachable!();
}

fn attach_inferior(pid: pid_t) -> Result<TrapInferior, Error> {
    match waitpid(pid, None) {
        Ok(WaitStatus::Stopped(pid, signal::SIGTRAP)) => return Ok(pid),
        Ok(_) => panic!("Unexpected stop in attach_inferior"),
        Err(e) => return Err(e)
    }
}

pub fn trap_inferior_exec(filename: &Path, args: &[&str]) -> Result<TrapInferior, Error> {
    loop {
        match fork() {
            Ok(Child)                      => exec_inferior(filename, args),
            Ok(Parent(pid))                => return attach_inferior(pid),
            Err(Error::Sys(errno::EAGAIN)) => continue,
            Err(e)                         => return Err(e)
        }
    }
}

pub fn trap_inferior_continue(inferior: TrapInferior) -> i8 {
    let pid = inferior;

    ptrace(PTRACE_CONT, pid, ptr::null_mut(), ptr::null_mut())
        .ok()
        .expect("Failed PTRACE_CONTINUE");

    match waitpid(pid, None) {
        Ok(WaitStatus::Exited(_pid, code)) => return code,
        Ok(_) => panic!("Unexpected stop in trap_inferior_continue"),
        Err(_) => panic!("Unhandled error in trap_inferior_continue")
    }
}
