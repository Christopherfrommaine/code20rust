use test1::fa::FA;

#[test]
fn canonical_label_test_1() {
    let fa1 = FA::from(vec!['a', 'b', 'c'], 2, vec![('a', 0, 'b'), ('b', 1, 'c'), ('c', 0, 'a'), ('c', 1, 'b')], 'a', vec!['a']);
    let fa2 = FA::from(vec!['B', 'C', 'A'], 2, vec![('B', 1, 'C'), ('A', 0, 'B'), ('C', 1, 'B'), ('C', 0, 'A')], 'A', vec!['A']);

    fa1.render_named_wl("fa1");
    fa2.render_named_wl("fa2");

    let fa1c = fa1.to_usize_fa().canonical_label();
    let fa2c = fa2.to_usize_fa().canonical_label();

    assert!(fa1c == fa2c);
    
}
