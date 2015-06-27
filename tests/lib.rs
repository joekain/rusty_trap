extern crate rusty_trap;

#[test]
fn it_can_exec () {
    let inferior = rusty_trap::trap_inferior_exec("./inferiors/twelve", &[]);
    assert_eq!(12, rusty_trap::trap_inferior_continue(inferior));
}
