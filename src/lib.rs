use std::ffi::CString;

pub type TrapInferior = i64;

pub fn trap_inferior_exec(filename: &str, args: &[&str]) -> TrapInferior {
    return 0;
}

pub fn trap_inferior_continue(inferior: TrapInferior) -> i8 {
    return 0;
}
