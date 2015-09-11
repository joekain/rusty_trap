use libc::pid_t;
use nix::sys::ptrace::*;
use nix::sys::ptrace::ptrace::*;
use std::ptr;
use libc::c_void;

pub mod inferior_pointer;

pub mod user {
    pub mod regs {
        // pub const R15: i64 = 0 * 8;
        // pub const R14: i64 = 1 * 8;
        // pub const R13: i64 = 2 * 8;
        // pub const R12: i64 = 3 * 8;
        // pub const RBP: i64 = 4 * 8;
        // pub const RBX: i64 = 5 * 8;
        // pub const R11: i64 = 6 * 8;
        // pub const R10: i64 = 7 * 8;
        // pub const R9:  i64 = 8 * 8;
        // pub const R8:  i64 = 9 * 8;
        // pub const RAX: i64 = 10 * 8;
        // pub const RCX: i64 = 11 * 8;
        // pub const RDX: i64 = 12 * 8;
        // pub const RSI: i64 = 13 * 8;
        // pub const RDI: i64 = 14 * 8;
        // pub const ORIG_RAX: i64 = 15 * 8;
        pub const RIP: i64 = 16 * 8;
        // pub const CS:  i64 = 17 * 8;
        // pub const EFLAGS: i64 = 18 * 8;
        // pub const RSP: i64 = 19 * 8;
        // pub const SS:  i64 = 20 * 8;
        // pub const FS_BASE: i64 = 21 * 8;
        // pub const GS_BASE: i64 = 22 * 8;
        // pub const DS:  i64 = 23 * 8;
        // pub const ES:  i64 = 24 * 8;
        // pub const FS:  i64 = 25 * 8;
        // pub const GS:  i64 = 26 * 8;
    }
}

pub fn trace_me() -> () {
    ptrace(PTRACE_TRACEME, 0, ptr::null_mut(), ptr::null_mut())
        .ok()
        .expect("Failed PTRACE_TRACEME");
}

pub fn get_instruction_pointer(pid: pid_t) -> inferior_pointer::InferiorPointer {
    let raw = ptrace(PTRACE_PEEKUSER, pid, user::regs::RIP as * mut c_void, ptr::null_mut())
        .ok()
        .expect("Failed PTRACE_PEEKUSER");
    inferior_pointer::InferiorPointer(raw as u64)
}

pub fn set_instruction_pointer(pid: pid_t, ip: inferior_pointer::InferiorPointer) -> () {
    ptrace(PTRACE_POKEUSER, pid, user::regs::RIP as * mut c_void, ip.as_voidptr())
        .ok()
        .expect("Failed PTRACE_POKEUSER");
}

pub fn cont(pid: pid_t) -> () {
    ptrace(PTRACE_CONT, pid, ptr::null_mut(), ptr::null_mut())
        .ok()
        .expect("Failed PTRACE_CONTINUE");
}

pub fn peek_text(pid: pid_t, address: inferior_pointer::InferiorPointer) -> i64 {
    ptrace(PTRACE_PEEKTEXT, pid, address.as_voidptr(), ptr::null_mut())
        .ok()
        .expect("Failed PTRACE_PEEKTEXT")
}

pub fn poke_text(pid: pid_t, address: inferior_pointer::InferiorPointer, value: i64) -> () {
    ptrace(PTRACE_POKETEXT, pid, address.as_voidptr(), value as * mut c_void)
        .ok()
        .expect("Failed PTRACE_POKETEXT");
}

pub fn single_step(pid: pid_t) -> () {
    ptrace(PTRACE_SINGLESTEP, pid, ptr::null_mut(), ptr::null_mut())
        .ok()
        .expect("Failed PTRACE_SINGLESTEP");
}
