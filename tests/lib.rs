extern crate rusty_trap;
use std::path::Path;

use rusty_trap::inferior::TrapData;

#[test]
fn it_can_exec() {
    let data = TrapData::new(Path::new("./target/debug/twelve"));
    let mut inferior = rusty_trap::trap_inferior_exec(&data, &[]).unwrap();
    let (_inferior, exit_code) = rusty_trap::trap_inferior_continue(&mut inferior, |_, _| {});
    assert_eq!(12, exit_code);
}

#[test]
fn it_can_set_breakpoints() {
    let mut breakpoint_count: i32 = 0;

    let data = TrapData::new(Path::new("./target/debug/twelve"));
    let mut inferior = rusty_trap::trap_inferior_exec(&data, &[]).unwrap();
    let expected_pid = inferior.pid;
    let (inferior, bp) = rusty_trap::trap_inferior_set_breakpoint(&mut inferior, "twelve::main");
    let (_, _) = rusty_trap::trap_inferior_continue(inferior, |passed_inferior, passed_bp| {
        assert_eq!(passed_inferior.pid, expected_pid);
        assert_eq!(passed_bp, bp);
        breakpoint_count += 1;
    });

    assert_eq!(breakpoint_count, 1);
}

#[test]
fn it_can_handle_a_breakpoint_more_than_once() {
    let mut breakpoint_count: i32 = 0;

    let data = TrapData::new(Path::new("./target/debug/loop"));
    let mut inferior = rusty_trap::trap_inferior_exec(&data, &[]).unwrap();
    let expected_pid = inferior.pid;
    let (inferior, bp) = rusty_trap::trap_inferior_set_breakpoint(&mut inferior, "loop::foo");
    rusty_trap::trap_inferior_continue(inferior, |passed_inferior, passed_bp| {
        assert_eq!(passed_inferior.pid, expected_pid);
        assert_eq!(passed_bp, bp);
        breakpoint_count += 1;
    });

    assert_eq!(breakpoint_count, 5);
}

#[test]
fn it_can_handle_more_than_one_breakpoint() {
    let mut bp_main_count: i32 = 0;
    let mut bp_foo_count: i32 = 0;

    let  data = TrapData::new(Path::new("./target/debug/loop"));
    let mut inferior = rusty_trap::trap_inferior_exec(&data, &[]).unwrap();
    let expected_pid = inferior.pid;
    let (inferior, bp_main) = rusty_trap::trap_inferior_set_breakpoint(&mut inferior, "loop::main");
    let (inferior, bp_foo) = rusty_trap::trap_inferior_set_breakpoint(inferior, "loop::foo");
    let (_, _) = rusty_trap::trap_inferior_continue(inferior, |passed_inferior, passed_bp| {
        assert_eq!(passed_inferior.pid, expected_pid);
        if passed_bp == bp_main {
            bp_main_count += 1;
        } else if passed_bp == bp_foo {
            bp_foo_count += 1;
        } else {
            panic!(
                "Unexpected breakpoint {} encountered.  Expected {} or {}",
                passed_bp, bp_main, bp_foo
            );
        }
    });

    assert_eq!(bp_main_count, 1);
    assert_eq!(bp_foo_count, 5);
}
