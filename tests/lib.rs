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
    let bp = rusty_trap::trap_inferior_set_breakpoint(inferior, 0x0000555555558800);
    rusty_trap::trap_inferior_continue(inferior, &mut |passed_inferior, passed_bp| {
        assert_eq!(passed_inferior, inferior);
        assert_eq!(passed_bp, bp);
        breakpoint_count += 1;
    });

    assert_eq!(breakpoint_count, 1);
}

#[test]
fn it_can_handle_a_breakpoint_more_than_once () {
    let mut breakpoint_count: i32 = 0;

    let inferior = rusty_trap::trap_inferior_exec(Path::new("./target/debug/loop"), &[])
        .unwrap();
    let bp = rusty_trap::trap_inferior_set_breakpoint(inferior, 0x5555555587b0);
    rusty_trap::trap_inferior_continue(inferior, &mut |passed_inferior, passed_bp| {
        assert_eq!(passed_inferior, inferior);
        assert_eq!(passed_bp, bp);
        breakpoint_count += 1;
    });

    assert_eq!(breakpoint_count, 5);
}

// #[test]
// fn it_can_handle_multiple_breakpoints () {
//     let mut foo_count:  i32 = 0;
//     let mut main_count: i32 = 0;
//
//     let inferior = rusty_trap::trap_inferior_exec(Path::new("./target/debug/loop"), &[])
//         .unwrap();
//     let bp_foo  = rusty_trap::trap_inferior_set_breakpoint(inferior, 0x5555555587b0);
//     let bp_main = rusty_trap::trap_inferior_set_breakpoint(inferior, 0x555555558bd0);
//     rusty_trap::trap_inferior_continue(inferior, &mut |passed_inferior, bp_passed| {
//         assert_eq!(passed_inferior, inferior);
//         if bp_passed == bp_foo {
//             foo_count  += 1
//         } else if bp_passed == bp_main {
//              main_count += 1
//         } else {
//             panic!("Unknown breakpoint {}", bp_passed)
//         }
//     });
//
//     assert_eq!(foo_count, 5);
//     assert_eq!(main_count, 5)
// }
