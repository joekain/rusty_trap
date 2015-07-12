extern crate rusty_trap;
use std::path::Path;

#[test]
fn it_can_exec () {
    let inferior = rusty_trap::trap_inferior_exec(Path::new("./target/debug/twelve"),&[])
        .unwrap();
    assert_eq!(12, rusty_trap::trap_inferior_continue(inferior, &mut |_, _| {}));
}

#[test]
fn it_can_set_breakpoints () {
    let mut breakpoint_count: i32 = 0;

    let inferior = rusty_trap::trap_inferior_exec(Path::new("./target/debug/twelve"), &[])
        .unwrap();
    let bp = rusty_trap::trap_inferior_set_breakpoint(inferior, 0x555555559000);
    rusty_trap::trap_inferior_continue(inferior, &mut |passed_inferior, passed_bp| {
        assert_eq!(passed_inferior, inferior);
        assert_eq!(passed_bp, bp);
        breakpoint_count += 1;
    });

    assert_eq!(breakpoint_count, 1);
}
