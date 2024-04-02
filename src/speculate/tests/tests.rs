use speculate::*;

#[tokio::test]
async fn test_spec() {
    assert!(spec(|| 2 + 2, || 4, |x| x + 2).await == 6);
    assert!(spec(|| 2 + 2, || 1, |x| x + 2).await == 6);
}
