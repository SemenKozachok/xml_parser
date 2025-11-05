use xml_parser::XmlNode;
use anyhow::Result;

#[test]
fn parses_simple_file() -> Result<()> {
    let path = "tests/samples/simple.txt";
    let node = XmlNode::from_path(&path)
        .map_err(|e| anyhow::anyhow!("failed to parse {:?}: {}", path, e))?;

    assert_eq!(node.name, "root");
    assert_eq!(node.children.len(), 1);
    assert_eq!(node.children[0].name, "item");
    assert_eq!(node.children[0].content, "Hello there");
    Ok(())
}

#[test]
fn detects_invalid_file() -> Result<()> {
    let path = "tests/samples/invalid.txt";
    let result = XmlNode::from_path(&path);
    assert!(result.is_err(), "expected error for invalid XML");
    Ok(())
}

#[test]
fn fails_if_file_missing() -> Result<()> {
    let path = "tests/samples/missing.xml";
    let result = XmlNode::from_path(&path);
    assert!(result.is_err(), "expected error for missing file");
    Ok(())
}

#[test]
fn parses_2names_file() -> Result<()> {
    let path = "tests/samples/2names.xml";
    let node = XmlNode::from_path(&path)
        .map_err(|e| anyhow::anyhow!("failed to parse {:?}: {}", path, e))?;

    let names = node.get_nodes("name")
                    .iter()
                    .map(|node| node.content.clone()).collect::<Vec<String>>();

    assert_eq!(names, vec!["bio", ""]);
    Ok(())
}

#[test]
fn parses_5names_file() -> Result<()> {
    let path = "tests/samples/5names.xml";
    let node = XmlNode::from_path(&path)
        .map_err(|e| anyhow::anyhow!("failed to parse {:?}: {}", path, e))?;

    let names = node.get_nodes("name")
                    .iter()
                    .map(|node| node.content.clone()).collect::<Vec<String>>();

    assert_eq!(
        names,
        vec!["rubber", "metal", "plastic", "timestamp", "timestamp"]
    );
    Ok(())
}
#[test]
fn prints_visual_tree() -> Result<()> {
    let path = "tests/samples/2names.xml";
    let node = XmlNode::from_path(&path)
        .map_err(|e| anyhow::anyhow!("failed to parse {:?}: {}", path, e))?;

    println!("\n====== Parsed XML Tree =====");
    println!("{}", node);

    Ok(())
}


