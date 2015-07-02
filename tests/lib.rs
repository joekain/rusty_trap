extern crate rusty_trap;

#[test]
fn it_can_exec () {
    let inferior = rusty_trap::trap_inferior_exec("./target/debug/twelve", &[]).unwrap();
    assert_eq!(12, rusty_trap::trap_inferior_continue(inferior));
}
