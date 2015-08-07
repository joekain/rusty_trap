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


pub type TrapInferior = pid_t;
pub type TrapBreakpoint = i32;

static mut original_breakpoint_word : i64 = 0;

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

pub fn trap_inferior_continue<F>(inferior: TrapInferior, callback: &mut F) -> i8
    where F: FnMut(TrapInferior, TrapBreakpoint) -> () {

    let pid = inferior;

    loop {
        ptrace_util::cont(pid);

        match waitpid(pid, None) {
            Ok(WaitStatus::Exited(_pid, code)) => return code,
            Ok(WaitStatus::Stopped(_pid, signal::SIGTRAP)) =>
                handle_breakpoint(inferior, callback),
            Ok(_) => panic!("Unexpected stop in trap_inferior_continue"),
            Err(_) => panic!("Unhandled error in trap_inferior_continue")
        }
    }
}

fn handle_breakpoint<F>(inferior: TrapInferior,  mut callback: &mut F) -> ()
    where F: FnMut(TrapInferior, TrapBreakpoint) -> () {

    let original = unsafe { original_breakpoint_word };
    let target_address = ptrace_util::get_instruction_pointer(inferior) - 1;
    ptrace_util::poke_text(inferior, target_address, original);

    callback(inferior, 0);

    ptrace_util::set_instruction_pointer(inferior, target_address);
}

pub fn trap_inferior_set_breakpoint(inferior: TrapInferior, location: usize) -> TrapBreakpoint {
    let aligned_address = location & !0x7usize;

    let original = ptrace_util::peek_text(inferior, aligned_address);
    let shift = (location - aligned_address) * 8;
    let mut modified = original;
    modified &= !0xFFi64 << shift;
    modified |= 0xCCi64 << shift;
    ptrace_util::poke_text(inferior, location, modified);

    unsafe { original_breakpoint_word = original as i64; }

    0
}
