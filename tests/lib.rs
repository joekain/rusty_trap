extern crate rusty_trap;
use std::path::Path;

const ADDRESS_OF_MAIN: u64 = 0x55555555b9f4;
const ADDRESS_OF_FOO: u64 = 0x55555555b9e0;

#[test]
fn it_can_exec() {
    let inferior = rusty_trap::trap_inferior_exec(Path::new("./target/debug/twelve"), &[]).unwrap();
    assert_eq!(
        12,
        rusty_trap::trap_inferior_continue(inferior, &mut |_, _| {})
    );
}

#[test]
fn it_can_set_breakpoints() {
    let mut breakpoint_count: i32 = 0;

    let inferior = rusty_trap::trap_inferior_exec(Path::new("./target/debug/twelve"), &[]).unwrap();
    let bp = rusty_trap::trap_inferior_set_breakpoint(inferior, 0x000055555555b821);
    rusty_trap::trap_inferior_continue(inferior, &mut |passed_inferior, passed_bp| {
        assert_eq!(passed_inferior, inferior);
        assert_eq!(passed_bp, bp);
        breakpoint_count += 1;
    });

    assert_eq!(breakpoint_count, 1);
}

#[test]
fn it_can_handle_a_breakpoint_more_than_once() {
    let mut breakpoint_count: i32 = 0;

    let inferior = rusty_trap::trap_inferior_exec(Path::new("./target/debug/loop"), &[]).unwrap();
    let bp = rusty_trap::trap_inferior_set_breakpoint(inferior, ADDRESS_OF_FOO);
    rusty_trap::trap_inferior_continue(inferior, &mut |passed_inferior, passed_bp| {
        assert_eq!(passed_inferior, inferior);
        assert_eq!(passed_bp, bp);
        breakpoint_count += 1;
    });

    assert_eq!(breakpoint_count, 5);
}

#[test]
fn it_can_handle_more_than_one_breakpoint() {
    let mut bp_main_count: i32 = 0;
    let mut bp_foo_count: i32 = 0;

    let inferior = rusty_trap::trap_inferior_exec(Path::new("./target/debug/loop"), &[]).unwrap();
    let bp_main = rusty_trap::trap_inferior_set_breakpoint(inferior, ADDRESS_OF_MAIN);
    let bp_foo = rusty_trap::trap_inferior_set_breakpoint(inferior, ADDRESS_OF_FOO);
    rusty_trap::trap_inferior_continue(inferior, &mut |passed_inferior, passed_bp| {
        assert_eq!(passed_inferior, inferior);
        if passed_bp == bp_main {
            bp_main_count += 1;
        } else if passed_bp == bp_foo {
            bp_foo_count += 1;
        }
    });

    assert_eq!(bp_main_count, 1);
    assert_eq!(bp_foo_count, 5);
}
