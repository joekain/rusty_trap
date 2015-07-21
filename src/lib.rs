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
use libc::c_void;


pub type TrapInferior = pid_t;
pub type TrapBreakpoint = i32;

static mut original_breakpoint_word : i64 = 0;

mod ffi {
    use libc::{c_int, c_long};

    extern {
        pub fn personality(persona: c_long) -> c_int;
    }
}

fn ptrace_util_get_instruction_pointer(pid: pid_t pid) -> usize
{
  uintptr_t offset = offsetof(struct user, regs.rip);
  return ptrace(PTRACE_PEEKUSER, pid, offset, ptr::null_mut());
}


fn ptrace_util_set_instruction_pointer(pid: pid_t pid, ip: usize) -> ()
{
  uintptr_t offset = offsetof(struct user, regs.rip);
  int result = ptrace(PTRACE_POKEUSER, pid, offset, ip);
  if (result != 0) {
    perror("ptrace_util_set_instruction_pointer: ");
    abort();
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

pub fn trap_inferior_continue<F>(inferior: TrapInferior, callback: &mut F) -> i8
    where F: FnMut(TrapInferior, TrapBreakpoint) -> () {

    let pid = inferior;

    loop {
        ptrace(PTRACE_CONT, pid, ptr::null_mut(), ptr::null_mut())
            .ok()
            .expect("Failed PTRACE_CONTINUE");

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

    ptrace(PTRACE_POKETEXT, inferior, target_address, original as * mut c_void)
        .ok()
        .expect("Failed PTRACE_POKETEXT");

    callback(inferior, 0)
}

pub fn trap_inferior_set_breakpoint(inferior: TrapInferior, location: usize) -> TrapBreakpoint {
    let target_address = location as * mut c_void;
    let aligned_address = location & !0x7usize;

    let original = ptrace(PTRACE_PEEKTEXT, inferior, aligned_address as * mut c_void, ptr::null_mut())
        .ok()
        .expect("Failed PTRACE_PEEKTEXT");

    let shift = (location - aligned_address) * 8;
    let mut modified = original as usize;
    modified &= !0xFFusize << shift;
    modified |= 0xCCusize << shift;

    ptrace(PTRACE_POKETEXT, inferior, target_address, modified as * mut c_void)
        .ok()
        .expect("Failed PTRACE_POKETEXT");

    unsafe { original_breakpoint_word = original as i64; }

    0
}
