use orbit::scan::progress::Progress;

#[test]
fn progress_buffers_and_drains() {
    let p = Progress::new(true);
    p.note("stage one");
    p.note("stage two");
    let drained = p.drain();
    assert_eq!(drained, vec!["stage one".to_string(), "stage two".to_string()]);
    // After drain, buffer should be empty
    let drained_again = p.drain();
    assert!(drained_again.is_empty());
}
