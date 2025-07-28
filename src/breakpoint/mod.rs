use inferior::*;
use nix::sys::signal;
use nix::sys::wait::*;
use nix::unistd::Pid;
use ptrace_util::*;
use object::{Object, ObjectSymbol};
use rustc_demangle::demangle;

#[derive(Copy, Clone)]
pub struct Breakpoint {
    shift: u64,
    target_address: InferiorPointer,
    aligned_address: InferiorPointer,
    original_breakpoint_word: i64,
}

pub type TrapBreakpoint = InferiorPointer;

fn step_over(inferior: &TrapInferior, bp: &Breakpoint) {
    poke_text(
        inferior.pid,
        bp.aligned_address,
        bp.original_breakpoint_word,
    );
    set_instruction_pointer(inferior.pid, bp.target_address);
    single_step(inferior.pid);
}

fn set(inferior: &TrapInferior, bp: &Breakpoint) {
    let mut modified = bp.original_breakpoint_word;
    modified &= !(0xFFi64 << bp.shift);
    modified |= 0xCCi64 << bp.shift;
    poke_text(inferior.pid, bp.aligned_address, modified);
}

fn find_breakpoint_matching_inferior_instruction_pointer<'a>(inf: &'a TrapInferior) -> Option<&'a Breakpoint> {
    let InferiorPointer(ip) = get_instruction_pointer(inf.pid);
    let ip = InferiorPointer(ip - 1);
    return inf.breakpoints.get(&ip);
}

pub fn handle<F>(inferior: &mut TrapInferior, callback: &mut F) -> InferiorState
where
    F: FnMut(&TrapInferior, TrapBreakpoint),
{
    let bp = find_breakpoint_matching_inferior_instruction_pointer(inferior)
        .expect("Could not find breakpoint");

    match inferior.state {
        InferiorState::Running => (),
        _ => panic!("Unhandled error in breakpoint::handle"),
    }
    callback(inferior, bp.target_address);
    step_over(inferior, bp);
    return match waitpid(Pid::from_raw(inferior.pid), None) {
        Ok(WaitStatus::Stopped(_pid, signal::SIGTRAP)) => {
            set(inferior, bp);
            cont(inferior.pid);
            InferiorState::Running
        }
        Ok(WaitStatus::Exited(_pid, _code)) => InferiorState::Running,
        Ok(WaitStatus::Stopped(_pid, signal)) => {
            panic!(
                "Unexpected stop on signal {} in breakpoint::handle.  State: {}",
                signal, inferior.state as i32
            )
        }
        Ok(_) => panic!("Unexpected stop in breakpoint::handle"),
        Err(_) => panic!("Unhandled error in breakpoint::handle"),
    };
}

fn set_breakpoint_at_address<'a>(
    inferior: &'a mut TrapInferior<'a>,
    location: u64
) -> (&'a mut TrapInferior<'a>, TrapBreakpoint) {
    let aligned_address = location & !0x7u64;
    let target_address = InferiorPointer(location);
    inferior.breakpoints.insert(
        target_address,
        Breakpoint {
            shift: (location - aligned_address) * 8,
            aligned_address: InferiorPointer(aligned_address),
            target_address,
            original_breakpoint_word: peek_text(inferior.pid, InferiorPointer(aligned_address)),
        },
    );

    set(
        &inferior,
        inferior.breakpoints.get(&target_address).unwrap(),
    );

    (inferior, InferiorPointer(location))
}

pub fn trap_inferior_set_breakpoint<'a>(
    inferior: &'a mut TrapInferior<'a>,
    location: &str,
) -> (&'a mut TrapInferior<'a>, TrapBreakpoint) {
    let mut address: u64 = 0;

    for symbol in inferior.obj.symbols() {
	let name = format!("{:#}", demangle(symbol.name().unwrap()));
	let symbol_address = symbol.address();
	if name == location {
	    println!("Found symbol {name} at 0x{symbol_address:x}");
	    address = symbol_address + inferior.base_address;
	    break;
	}
    }

    return set_breakpoint_at_address(inferior, address);
}
