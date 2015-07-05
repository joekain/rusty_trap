extern crate rusty_trap;
use std::path::Path;

#[test]
fn it_can_exec () {
    let inferior = rusty_trap::trap_inferior_exec(Path::new("./target/debug/twelve"),&[])
        .unwrap();
    assert_eq!(12, rusty_trap::trap_inferior_continue(inferior));
}

#[test]
fn it_can_set_breakpoints () {
    let mut bp: rusty_trap::Breakpoint = 0;
    let mut breakpoint_count: i32 = 0;
    let mut inferior = 0;

    rusty_trap::trap_set_breakpoint_callback(|passed_inferior, passed_bp| {
        assert_eq!(passed_inferior, inferior);
        assert_eq!(passed_bp, bp);
        breakpoint_count += 1;
    }).unwrap();

    let inferior = rusty_trap::trap_inferior_exec("./target/debug/twelve", &[]).unwrap();
    bp = rusty_trap::trap_inferior_set_breakpoint(inferior, "main");
    trap_inferior_continue(inferior);

    assert_eq!(breakpoint_count, 1);
}
