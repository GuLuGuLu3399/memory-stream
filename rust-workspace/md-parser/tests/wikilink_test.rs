use md_parser::extract_wikilinks;

#[test]
fn test_extract_basic() {
    let md = "text [[Card A]] and [[Card B]] end";
    let result = extract_wikilinks(md);
    assert_eq!(result, vec!["Card A", "Card B"]);
}

#[test]
fn test_extract_dedup() {
    let md = "[[Duplicate]] [[Duplicate]]";
    let result = extract_wikilinks(md);
    assert_eq!(result, vec!["Duplicate"]);
}

#[test]
fn test_extract_spaces() {
    let md = "[[Card With Spaces]]";
    let result = extract_wikilinks(md);
    assert_eq!(result, vec!["Card With Spaces"]);
}

#[test]
fn test_extract_unicode() {
    let md = "[[Émoji 🎉]]";
    let result = extract_wikilinks(md);
    assert_eq!(result, vec!["Émoji 🎉"]);
}

#[test]
fn test_extract_empty() {
    let md = "";
    let result = extract_wikilinks(md);
    assert!(result.is_empty());
}

#[test]
fn test_extract_nested_brackets() {
    let md = "[[Outer [[Inner]]]]";
    let result = extract_wikilinks(md);
    // With pattern \[\[([^\]]+)\]\], it will match the first complete [[...]]
    // Since [^\]]+ doesn't match ], it will stop at the first ]
    // So [[Outer [[Inner]] will match, capturing "Outer [[Inner"
    assert_eq!(result, vec!["Outer [[Inner"]);
}

#[test]
fn test_extract_none() {
    let md = "No wikilinks here";
    let result = extract_wikilinks(md);
    assert!(result.is_empty());
}

#[test]
fn test_extract_self_reference() {
    let md = "[[Same Card]]";
    let result = extract_wikilinks(md);
    assert_eq!(result, vec!["Same Card"]);
}
