use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;
use std::{fs, io};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Grammar;

#[derive(Debug)]
pub struct XmlNode {
    pub name: String,
    pub content: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<XmlNode>,
}

impl std::fmt::Display for XmlNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_node(f, 0)
    }
}

impl XmlNode {
    pub fn from_path(path: &str) -> Result<Self, ParseError> {
        let data = fs::read_to_string(path)?;
        parse_xml(&data)
    }

    pub fn get_contents_of(&self, tag: &str) -> Option<&str> {
        if self.name == tag && !self.content.is_empty() {
            return Some(self.content.as_str());
        }

        for child in &self.children {
            if let Some(found) = child.get_contents_of(tag) {
                return Some(found);
            }
        }

        None
    }

    pub fn get_nodes(&self, tag: &str) -> Vec<&XmlNode> {
        let mut results = Vec::new();

        if self.name == tag {
            results.push(self);
        }

        for child in &self.children {
            results.extend(child.get_nodes(tag));
        }
        results
    }

fn display_node(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        let pad = "  ".repeat(indent);
        write!(f, "{}<{}", pad, self.name)?;

        for (k, v) in &self.attributes {
            write!(f, " {}=\"{}\"", k, v)?;
        }
        writeln!(f, ">")?;

        if !self.content.is_empty() {
            writeln!(f, "{}  {}", pad, self.content)?;
        }

        for child in &self.children {
            child.display_node(f, indent + 3)?;
        }

        writeln!(f, "{}</{}>", pad, self.name)
    }

}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Tag mismatch: opening tag <{opening}>, ending tag </{ending}>")]
    TagMismatch { opening: String, ending: String },

    #[error("Unexpected structure or syntax error in XML")]
    SyntaxError,

    #[error("File I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Unexpected Internal Error: {message}\nIf you see this error, something went wrong :/")]
    InternalError{message: String},
}

pub fn parse_xml(input: &str) -> Result<XmlNode, ParseError> {
    let mut parsed = Grammar::parse(Rule::xml, input)
        .map_err(|_| ParseError::SyntaxError)?;

    let root = parsed.next().ok_or(ParseError::SyntaxError)?;

    let start_element = root
        .into_inner()
        .find(|p| p.as_rule() == Rule::element)
        .ok_or(ParseError::SyntaxError)?;

    parse_element(start_element)
}


fn parse_element(element: pest::iterators::Pair<Rule>) -> Result<XmlNode, ParseError> {
    let mut inner = element.into_inner();
    let pair = inner.next().ok_or(ParseError::SyntaxError)?;

    match pair.as_rule() {
        Rule::full_element => {
            let mut inner = pair.into_inner();
            let opening = inner.next().ok_or(ParseError::SyntaxError)?;
            let (name_open, attrs) = parse_opening_tag(opening)?;

            let mut children = Vec::new();
            let mut content = String::new();

            for item in inner {
                match item.as_rule() {
                    Rule::content => content.push_str(item.as_str().trim()),
                    Rule::element => children.push(parse_element(item)?),
                    Rule::closing_tag => {
                        let name_close = item.into_inner().next().unwrap().as_str().to_string();
                        if name_close != name_open {
                            return Err(ParseError::TagMismatch {
                                opening: name_open,
                                ending: name_close,
                            });
                        }
                        return Ok(XmlNode {
                            name: name_open,
                            attributes: attrs,
                            content,
                            children,
                        });
                    }
                    _ => {}
                }
            }
            Err(ParseError::SyntaxError)
        }

        Rule::empty_element_tag => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let attrs = parse_attributes(inner);
            Ok(XmlNode {
                name,
                attributes: attrs,
                content: String::new(),
                children: Vec::new(),
            })
        }

        Rule::comment => Ok(XmlNode {
            name: "#comment".to_string(),
            attributes: Vec::new(),
            content: pair.as_str().to_string(),
            children: Vec::new(),
        }),

        _ => Err(ParseError::InternalError {
            message: format!("Unexpected rule: {:?}", pair.as_rule()),
        }),
    }
}

fn parse_opening_tag(pair: pest::iterators::Pair<Rule>,
) -> Result<(String, Vec<(String, String)>), ParseError> {

    let mut inner = pair.into_inner().filter(|p| p.as_rule() != Rule::WHITESPACE);
    let name = inner.next().ok_or(ParseError::SyntaxError)?.as_str().to_string();
    let attrs = parse_attributes(inner);
    Ok((name, attrs))
}

fn parse_attributes<'a>(
    pairs: impl Iterator<Item = pest::iterators::Pair<'a, Rule>>,
) -> Vec<(String, String)> {
    let mut attributes = Vec::new();
    for attr in pairs {
        if attr.as_rule() == Rule::attribute {
            let mut parts = attr.into_inner();
            let key = parts.next().unwrap().as_str().to_string();
            let value = parts.next().unwrap().as_str().trim_matches('"').to_string();
            attributes.push((key, value));
        }
    }
    attributes
}


#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(input: &str) -> XmlNode {
        parse_xml(input).expect("expected valid XML")
    }

    fn parse_err(input: &str) -> ParseError {
        parse_xml(input).unwrap_err()
    }


    #[test]
    fn parses_empty_element() {
        let xml = "<root></root>";
        let result = parse_ok(xml);

        assert_eq!(result.name, "root");
        assert_eq!(result.content, "");
        assert!(result.children.is_empty());
    }

    #[test]
    fn parses_simple_element() {
        let xml = "<root>hello</root>";
        let result = parse_ok(xml);

        assert_eq!(result.name, "root");
        assert_eq!(result.content, "hello");
        assert!(result.children.is_empty());
    }

    #[test]
    fn ignores_whitespace_in_content() {
        let xml = "<root>   s p a c e d   </root>";
        let result = parse_ok(xml);

        assert_eq!(result.content, "s p a c e d");
    }

    #[test]
    fn parses_nested_elements() {
        let xml = "<root><a>1</a><b>2</b></root>";
        let result = parse_ok(xml);

        assert_eq!(result.name, "root");
        assert_eq!(result.children.len(), 2);
        assert_eq!(result.children[0].name, "a");
        assert_eq!(result.children[0].content, "1");
        assert_eq!(result.children[1].name, "b");
        assert_eq!(result.children[1].content, "2");
    }

    #[test]
    fn parses_nested_elements_with_content() {
        let xml = "<root>Content<a>1</a><b>2</b></root>";
        let result = parse_ok(xml);
        assert_eq!(result.name, "root");
        assert_eq!(result.children.len(), 2);
        assert_eq!(result.content, "Content");
        assert_eq!(result.children[0].name, "a");
        assert_eq!(result.children[0].content, "1");
        assert_eq!(result.children[1].name, "b");
        assert_eq!(result.children[1].content, "2");
    }

    #[test]
    fn detects_empty_input() {
        let xml = "";
        match parse_err(xml) {
            ParseError::SyntaxError => {}
            _ => panic!("expected SyntaxError"),
        }
    }

    #[test]
    fn detects_tag_mismatch() {
        let xml = "<root><a>1</b></root>";
        match parse_err(xml) {
            ParseError::TagMismatch { opening, ending } => {
                assert_eq!(opening, "a");
                assert_eq!(ending, "b");
            }
            _ => panic!("expected TagMismatch error"),
        }
    }

    #[test]
    fn detects_unexpected_structure() {
        let xml = "<root><a></root>";
        match parse_err(xml) {
            ParseError::SyntaxError => {}
            _ => panic!("expected SyntaxError"),
        }
    }
    
    #[test]
    fn detects_file_error() {
        let result = XmlNode::from_path("nonexistent.xml");
        match result {
            Err(ParseError::IoError(_)) => {}
            _ => panic!("expected IoError for missing file"),
        }
    }
    #[test]
    fn parses_comment() {
        let xml = "<root><!-- this is a comment --></root>";
        let node = parse_ok(xml);

        assert_eq!(node.name, "root");
        assert_eq!(node.children[0].content, "<!-- this is a comment -->");
        assert_eq!(node.content, "");
    }

    #[test]
    fn parses_declaration() {
        let xml = r#"<?xml ?><root></root>"#;
        let node = parse_ok(xml);

        assert_eq!(node.name, "root");
        assert!(node.children.is_empty());
    }

    #[test]
    fn parses_empty_element_tag() {
        let xml = "<root><empty /></root>";
        let node = parse_ok(xml);

        assert_eq!(node.name, "root");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].name, "empty");
        assert!(node.children[0].children.is_empty());
        assert_eq!(node.children[0].content, "");
    }

    #[test]
    fn parses_element_with_attribute() {
        let xml = r#"<root><a attr="value"></a></root>"#;
        let node = parse_ok(xml);

        assert_eq!(node.name, "root");
        assert_eq!(node.children[0].attributes[0].0, "attr");
        assert_eq!(node.content, "");
    }

    #[test]
    fn parses_element_with_attributes() {
        let xml = r#"<root><a attr="value" id="2"></a></root>"#;
        let node = parse_ok(xml);

        assert_eq!(node.name, "root");
        assert_eq!(node.children[0].attributes[0].0, "attr");
        assert_eq!(node.children[0].attributes[1].1, "2");
        assert_eq!(node.content, "");
    }
}
