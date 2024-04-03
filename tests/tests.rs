use speculate::*;

#[test]
fn test_spec() {
    assert!(spec(|| 2 + 2, || 4, |x| x + 2) == 6);
    assert!(spec(|| 2 + 2, || 1, |x| x + 2) == 6);
}
