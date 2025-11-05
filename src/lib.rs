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
        writeln!(f, "{}<{}>", pad, self.name)?;

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

    let root = parsed
        .next()
        .ok_or(ParseError::SyntaxError)?;

    let element = root.into_inner().next().unwrap();
    parse_element(element)
}

fn parse_element(element: pest::iterators::Pair<Rule>) -> Result<XmlNode, ParseError> {
    let mut inner = element.into_inner();

    let opening = inner
        .next()
        .ok_or(ParseError::SyntaxError)?;
    let name_open = opening.into_inner().next().unwrap().as_str().to_string();

    let mut children = Vec::new();
    let mut content = String::new();

    for item in inner {
        match item.as_rule() {
            Rule::content => {
                content.push_str(item.as_str().trim());
            }
            Rule::element => {
                let child = parse_element(item)?;
                children.push(child);
            }
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
                    content,
                    children,
                });
            }
            _ => return Err(ParseError::InternalError{message: "Got unexpected token".to_string()}),
        }
    }
    Err(ParseError::InternalError{message: "Went out of recursion loop incorrectly".to_string()})
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
}
