# XML Parser

This project implements a simple XML parser in Rust, based on the [`pest`](https://pest.rs/) parsing library.  
It converts input into tree consisting of XmlNode`s and implements DFS search algorithm as well as display of a parsed tree. The parser can be used both as a library and via a command-line interface (CLI).

## Overview

The parser reads files (any file type that can be read as string and contains valid xml structure), validates tag structure, and constructs the tree recursively. The parsing uses pest grammar rules in grammar.pest file. CLI allows user to parse given file, find content of given tag or find contents of all elements of given tag.

The parser supports:
- Nested and sequential XML elements.
- Automatic whitespace trimming in text nodes.
- Error handling for tag mismatches, syntax violations, and file reading issues issues.

The CLI implements:
- Access to nodes and their contents (`-get`, `-get_all`).
- Formatted visual output of parsed XML trees through the `Display` trait.
- Error handling for wrong commands, incorrect files or parsing errors.
- Credits and help commands.

The tests include:
- unit tests for every grammar rule and error in lib.rs
- integrated tests for parsing files in tests/parse_file.rs
- integrated tests for CLI in tests/parse_file.rs

## Grammar

Description of every rule:

| Rule | Description |
|------|--------------|
| **xml** | main rule that represents the whole xml document. |
| **element** | single element of the tree(node) that might have other nodes or content inside it|
| **opening_tag** | `<tag>` begining of an element |
| **closing_tag** | `</tag>` the end of an element |
| **content** | Matches raw text between tags, trimming whitespaces and lineskips like \n. |
| **tag_name** | name of the tag(element) between <>. |

## Error handling

- `TagMismatch` — opening and closing tags do not match.
- `SyntaxError` — the document structure is invalid for XML.
- `IoError` — failure to read from a file.
- `InternalError` — Error that should not happen (encountering it means you found a bug, I am sorry :/ ).

## Tree Structure

The output is a parent Node `XmlNode` of the tree, each node has name , content(possibly empty) and Vector of child Nodes(possibly empty).

It is constructed by recursively calling function that parses an element each time it finds element rule and returning Node each time it finds closing_tag.

