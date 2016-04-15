use ptrace_util::*;
use inferior::*;

pub type TrapBreakpoint = i32;

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


fn step_over(inferior: &TrapInferior, bp: Breakpoint) -> () {
    let pid = inferior.pid;
    poke_text(pid, bp.aligned_address, bp.original_breakpoint_word);
    set_instruction_pointer(pid, bp.target_address);
    single_step(pid);
}

fn set(inferior: &TrapInferior, bp: Breakpoint) -> () {
    let mut modified = bp.original_breakpoint_word;
    modified &= !0xFFi64 << bp.shift;
    modified |= 0xCCi64 << bp.shift;
    poke_text(inferior.pid, bp.aligned_address, modified);
}

pub fn handle<F>(inf: &TrapInferior,  mut callback: &mut F) -> InferiorState
    where F: FnMut(&TrapInferior, TrapBreakpoint) -> () {

    let bp = unsafe { global_breakpoint };
    match inf.state {
        InferiorState::Running => {
            callback(&inf, 0);
            step_over(&inf, bp);
            InferiorState::SingleStepping
        },
        InferiorState::SingleStepping => {
            set(&inf, bp);
            cont(inf.pid);
            InferiorState::Running
        },
        _ => panic!("Unsupported breakpoint encountered during supported inferior state")
    }
}

pub fn trap_inferior_set_breakpoint(inferior: TrapInferior, location: u64) -> TrapBreakpoint {
    let aligned_address = location & !0x7u64;
    let bp = Breakpoint {
        shift : (location - aligned_address) * 8,
        aligned_address: InferiorPointer(aligned_address),
        target_address: InferiorPointer(location),
        original_breakpoint_word: peek_text(inferior.pid, InferiorPointer(aligned_address))
    };

    set(&inferior, bp);

    unsafe {
        global_breakpoint = bp;
    }

    0
}
