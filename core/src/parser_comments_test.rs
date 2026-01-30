#[test]
fn test_parse_comments() {
    // Comment on its own line
    let src = "
        ; this is a comment
        (print 1)";
    let vals = parse(src).unwrap();
    assert_eq!(vals.len(), 1);
    if let Value::List(v) = &vals[0] {
        assert_eq!(v[0], Value::Symbol("print".to_string()));
    }

    // Inline comment
    let src2 = "(print 2) ; inline comment";
    let vals2 = parse(src2).unwrap();
    assert_eq!(vals2.len(), 1);

    // Comment between items
    let src3 = "(+ 1 ; comment\n 2)";
    let vals3 = parse(src3).unwrap();
    let list = &vals3[0];
    if let Value::List(v) = list {
        assert_eq!(v.len(), 3); // +, 1, 2
    }
}
