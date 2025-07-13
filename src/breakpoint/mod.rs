use inferior::*;
use ptrace_util::*;
use nix::sys::wait::*;
use nix::unistd::Pid;
use nix::sys::signal;

use crate::ptrace_util;

#[derive(Copy, Clone)]
struct Breakpoint {
    shift: u64,
    target_address: InferiorPointer,
    aligned_address: InferiorPointer,
    original_breakpoint_word: i64,
}

pub type TrapBreakpoint = InferiorPointer;

static mut GLOBAL_BREAKPOINTS: Vec<Breakpoint> = Vec::<Breakpoint>::new();

fn step_over(inferior: TrapInferior, bp: Breakpoint) {
    poke_text(inferior, bp.aligned_address, bp.original_breakpoint_word);
    set_instruction_pointer(inferior, bp.target_address);
    single_step(inferior);
}

fn set(inferior: TrapInferior, bp: Breakpoint) {
    let mut modified = bp.original_breakpoint_word;
    modified &= !(0xFFi64 << bp.shift);
    modified |= 0xCCi64 << bp.shift;
    poke_text(inferior, bp.aligned_address, modified);
}

fn find_breakpoint_matching_inferior_instruction_pointer(inf: Inferior) -> Option<Breakpoint> {
    let ip = ptrace_util::get_instruction_pointer(inf.pid).as_i64();
    unsafe { for bp in &GLOBAL_BREAKPOINTS {
	if bp.target_address.as_i64() == ip - 1 {
	    return Some(*bp)
	}
    } };
    None
}

pub fn handle<F>(inf: Inferior, callback: &mut F) -> InferiorState
where
    F: FnMut(TrapInferior, TrapBreakpoint),
{
    let inferior = inf.pid;
    let bp =
        find_breakpoint_matching_inferior_instruction_pointer(inf)
        .expect("Could not find breakpoint");

    match inf.state {
        InferiorState::Running => (),
        _ => panic!("Unhandled error in breakpoint::handle"),
    }
    callback(inferior, bp.target_address);
    step_over(inferior, bp);
    return match waitpid(Pid::from_raw(inf.pid), None) {
        Ok(WaitStatus::Stopped(_pid, signal::SIGTRAP)) => {
            set(inferior, bp);
            cont(inferior);
            InferiorState::Running
        }
        Ok(WaitStatus::Exited(_pid, _code)) => InferiorState::Running,
        Ok(WaitStatus::Stopped(_pid, signal)) => {
            panic!(
                "Unexpected stop on signal {} in breakpoint::handle.  State: {}",
                signal, inf.state as i32
            )
        }
        Ok(_) => panic!("Unexpected stop in breakpoint::handle"),
        Err(_) => panic!("Unhandled error in breakpoint::handle"),
    };
}

pub fn trap_inferior_set_breakpoint(inferior: TrapInferior, location: u64) -> TrapBreakpoint {
    let aligned_address = location & !0x7u64;
    let index = unsafe {
	GLOBAL_BREAKPOINTS.push(Breakpoint {
            shift: (location - aligned_address) * 8,
            aligned_address: InferiorPointer(aligned_address),
            target_address: InferiorPointer(location),
            original_breakpoint_word: peek_text(inferior, InferiorPointer(aligned_address)),
	});
	GLOBAL_BREAKPOINTS.len() - 1
    };

    set(inferior, unsafe {GLOBAL_BREAKPOINTS[index]});

    InferiorPointer(location)
}
