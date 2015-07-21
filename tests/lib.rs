extern crate rusty_trap;
use std::path::Path;

#[test]
fn it_can_exec () {
    let inferior = rusty_trap::trap_inferior_exec(Path::new("./target/debug/twelve"),&[])
        .unwrap();
    assert_eq!(12, rusty_trap::trap_inferior_continue(inferior));
}
