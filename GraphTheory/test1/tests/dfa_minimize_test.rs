use test1::fa::FA;

#[test]
fn dfa_minimize_test_1() {
    let fa = FA::from(vec![0, 1, 2], 2, vec![(0, 0, 0), (0, 1, 2), (2, 0, 2), (2, 1, 1), (1, 0, 0), (0, 0, 1)], 0, vec![0]);
    fa.bs_usize_to_dfa().render_named_wl("dfa_minimize_test_1>before");
    fa.bs_usize_to_dfa().dfa_minimize().render_named_wl("dfa_minimize_test_1>after");
}

#[test]
fn dfa_minimize_test_2() {
    let fa = FA::from(vec![0, 1, 2, 3], 2, vec![(0, 0, 1), (1, 0, 1), (1, 1, 2), (0, 0, 2), (2, 1, 3), (3, 1, 0)], 0, vec![0]);
    fa.bs_usize_to_dfa().render_named_wl("dfa_minimize_test_2>before");
    fa.bs_usize_to_dfa().dfa_minimize().render_named_wl("dfa_minimize_test_2>after");
}