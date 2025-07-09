use libc::pid_t;
use nix::sys::ptrace;
use nix::unistd::Pid;

use inferior::InferiorPointer;

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
    ptrace::traceme()
        .ok()
        .expect("Failed PTRACE_TRACEME");
}

pub fn get_instruction_pointer(pid: pid_t) -> InferiorPointer {
    let raw = ptrace::read_user(Pid::from_raw(pid), user::regs::RIP as ptrace::AddressType)
        .ok()
        .expect("Failed PTRACE_PEEKUSER");
    InferiorPointer(raw as u64)
}

pub fn set_instruction_pointer(pid: pid_t, ip: InferiorPointer) -> () {
    ptrace::write_user(Pid::from_raw(pid), user::regs::RIP as ptrace::AddressType, ip.as_i64())
        .ok()
        .expect("Failed PTRACE_POKEUSER");
}

pub fn cont(pid: pid_t) -> () {
    ptrace::cont(Pid::from_raw(pid), None)
        .ok()
        .expect("Failed PTRACE_CONTINUE");
}

pub fn peek_text(pid: pid_t, address: InferiorPointer) -> i64 {
    // From ptrace(2) regarding PTRACE_PEEKTEXT and PTRACE_PEEKDATA
    //   Linux does not have separate text and data address spaces,
    //   so these two operations are currently equivalent.
    // So use ptrace::read which is ptrace(PTRACE_PEEKDATA, ...)
    // An alterantive would be to use libc::ptrace.
    ptrace::read(Pid::from_raw(pid),  address.as_voidptr())
	.ok()
	.expect("Failed PTRACE_PEEKTEXT")
}

pub fn poke_text(pid: pid_t, address: InferiorPointer, value: i64) -> () {
    ptrace::write(Pid::from_raw(pid), address.as_voidptr(), value)
	.ok()
	.expect("Failed PTRACE_POKETEXT")
}

pub fn single_step(pid: pid_t) -> () {
    ptrace::step(Pid::from_raw(pid), None)
        .ok()
        .expect("Failed PTRACE_SINGLESTEP");
}
