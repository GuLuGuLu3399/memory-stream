use md_parser::extract_wikilinks;

#[test]
fn test_extract_basic() -> Result<(), Box<dyn std::error::Error>> {
    let md = "text [[Card A]] and [[Card B]] end";
    let result = extract_wikilinks(md)?;
    assert_eq!(result, vec!["Card A", "Card B"]);
    Ok(())
}

#[test]
fn test_extract_dedup() -> Result<(), Box<dyn std::error::Error>> {
    let md = "[[Duplicate]] [[Duplicate]]";
    let result = extract_wikilinks(md)?;
    assert_eq!(result, vec!["Duplicate"]);
    Ok(())
}

#[test]
fn test_extract_spaces() -> Result<(), Box<dyn std::error::Error>> {
    let md = "[[Card With Spaces]]";
    let result = extract_wikilinks(md)?;
    assert_eq!(result, vec!["Card With Spaces"]);
    Ok(())
}

#[test]
fn test_extract_unicode() -> Result<(), Box<dyn std::error::Error>> {
    let md = "[[Émoji 🎉]]";
    let result = extract_wikilinks(md)?;
    assert_eq!(result, vec!["Émoji 🎉"]);
    Ok(())
}

#[test]
fn test_extract_empty() -> Result<(), Box<dyn std::error::Error>> {
    let md = "";
    let result = extract_wikilinks(md)?;
    assert!(result.is_empty());
    Ok(())
}

#[test]
fn test_extract_nested_brackets() -> Result<(), Box<dyn std::error::Error>> {
    let md = "[[Outer [[Inner]]]]";
    let result = extract_wikilinks(md)?;
    // 非贪婪匹配 `.+?` 在遇到第一对 `]]` 时停止
    // 所以 `[[Outer [[Inner]]` 匹配，捕获 "Outer [[Inner"
    assert_eq!(result, vec!["Outer [[Inner"]);
    Ok(())
}

#[test]
fn test_extract_none() -> Result<(), Box<dyn std::error::Error>> {
    let md = "No wikilinks here";
    let result = extract_wikilinks(md)?;
    assert!(result.is_empty());
    Ok(())
}

#[test]
fn test_extract_self_reference() -> Result<(), Box<dyn std::error::Error>> {
    let md = "[[Same Card]]";
    let result = extract_wikilinks(md)?;
    assert_eq!(result, vec!["Same Card"]);
    Ok(())
}

#[test]
fn test_extract_parentheses_in_name() -> Result<(), Box<dyn std::error::Error>> {
    // 非贪婪匹配正确处理链接内部的括号和特殊字符
    let md = "[[矩阵(Matrix)特征值]]";
    let result = extract_wikilinks(md)?;
    assert_eq!(result, vec!["矩阵(Matrix)特征值"]);
    Ok(())
}

#[test]
fn test_extract_brackets_in_name() -> Result<(), Box<dyn std::error::Error>> {
    // 非贪婪匹配甚至能处理链接内部包含 ] 的情况
    let md = "[[some]thing]]";
    let result = extract_wikilinks(md)?;
    assert_eq!(result, vec!["some]thing"]);
    Ok(())
}
