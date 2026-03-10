use anyhow::Result;
use tree_sitter::{Language, Parser as TSParser, Node};
use crate::storage::Symbol;

pub struct Parser {
    parser: TSParser,
}

impl Parser {
    pub fn new() -> Result<Self> {
        let parser = TSParser::new();
        Ok(Self { parser })
    }

    pub fn parse_file(&mut self, source: &str, ext: &str, repo: &str, file_path: &str) -> Result<Vec<Symbol>> {
        let (lang_name, ts_lang) = match get_language_info(ext) {
            Some(info) => info,
            None => return Ok(Vec::new()),
        };

        self.parser.set_language(&ts_lang)?;
        let tree = self.parser.parse(source, None).ok_or_else(|| anyhow::anyhow!("Failed to parse"))?;
        
        let mut symbols = Vec::new();
        let source_bytes = source.as_bytes();
        self.walk_node(tree.root_node(), source_bytes, lang_name, repo, file_path, &mut symbols)?;
        
        Ok(symbols)
    }

    fn walk_node(
        &self,
        node: Node,
        source: &[u8],
        lang_name: &str,
        repo: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>
    ) -> Result<()> {
        let kind = node.kind();
        
        if let Some(symbol_kind) = get_symbol_kind(lang_name, kind) {
            if let Some(name) = extract_name(node, source) {
                let signature = extract_signature(node, source);
                let docstring = extract_docstring(node, source, lang_name);
                
                symbols.push(Symbol {
                    id: None,
                    repo: repo.to_string(),
                    file: file_path.to_string(),
                    name,
                    kind: symbol_kind.to_string(),
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    signature: Some(signature),
                    docstring,
                    embedding: None,
                });
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_node(child, source, lang_name, repo, file_path, symbols)?;
        }

        Ok(())
    }
}

fn get_language_info(ext: &str) -> Option<(&'static str, Language)> {
    match ext {
        "py" => Some(("python", tree_sitter_python::LANGUAGE.into())),
        "js" | "mjs" => Some(("javascript", tree_sitter_javascript::LANGUAGE.into())),
        "ts" => Some(("typescript", tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())),
        "tsx" => Some(("typescript", tree_sitter_typescript::LANGUAGE_TSX.into())),
        "go" => Some(("go", tree_sitter_go::LANGUAGE.into())),
        "rs" => Some(("rust", tree_sitter_rust::LANGUAGE.into())),
        "java" => Some(("java", tree_sitter_java::LANGUAGE.into())),
        "php" => Some(("php", tree_sitter_php::LANGUAGE_PHP.into())),
        "cs" => Some(("c_sharp", tree_sitter_c_sharp::LANGUAGE.into())),
        "c" => Some(("c", tree_sitter_c::LANGUAGE.into())),
        "cpp" | "h" | "cc" => Some(("cpp", tree_sitter_cpp::LANGUAGE.into())),
        "ex" | "exs" => Some(("elixir", tree_sitter_elixir::LANGUAGE.into())),
        "rb" => Some(("ruby", tree_sitter_ruby::LANGUAGE.into())),
        "vue" => Some(("vue", tree_sitter_vue_next::LANGUAGE.into())),
        _ => None,
    }
}

fn get_symbol_kind(lang: &str, node_kind: &str) -> Option<&'static str> {
    match lang {
        "python" => match node_kind {
            "function_definition" => Some("function"),
            "class_definition" => Some("class"),
            "decorated_definition" => Some("decorated"),
            _ => None,
        },
        "javascript" | "typescript" => match node_kind {
            "function_declaration" | "method_definition" | "arrow_function" | "function_expression" => Some("function"),
            "class_declaration" => Some("class"),
            "interface_declaration" => Some("class"),
            _ => None,
        },
        "go" => match node_kind {
            "function_declaration" => Some("function"),
            "method_declaration" => Some("method"),
            "type_declaration" => Some("class"),
            _ => None,
        },
        "rust" => match node_kind {
            "function_item" => Some("function"),
            "struct_item" | "enum_item" | "type_item" | "trait_item" | "impl_item" => Some("class"),
            _ => None,
        },
        "java" => match node_kind {
            "method_declaration" => Some("method"),
            "class_declaration" | "interface_declaration" | "enum_declaration" => Some("class"),
            _ => None,
        },
        "php" => match node_kind {
            "function_definition" => Some("function"),
            "class_declaration" => Some("class"),
            "method_declaration" => Some("method"),
            _ => None,
        },
        "c_sharp" => match node_kind {
            "method_declaration" => Some("method"),
            "class_declaration" | "interface_declaration" | "struct_declaration" => Some("class"),
            _ => None,
        },
        "c" => match node_kind {
            "function_definition" => Some("function"),
            "struct_specifier" | "enum_specifier" => Some("class"),
            _ => None,
        },
        "cpp" => match node_kind {
            "function_definition" => Some("function"),
            "class_specifier" | "struct_specifier" | "enum_specifier" => Some("class"),
            _ => None,
        },
        "elixir" => match node_kind {
            "def" | "defp" => Some("function"),
            "defmodule" => Some("class"),
            _ => None,
        },
        "ruby" => match node_kind {
            "method" => Some("method"),
            "class" | "module" => Some("class"),
            _ => None,
        },
        _ => None,
    }
}

fn extract_name(node: Node, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        let child = node.child(i).unwrap();
        if child.kind() == "identifier" || child.kind() == "name" || child.kind() == "type_identifier" {
            if let Ok(name) = std::str::from_utf8(&source[child.start_byte()..child.end_byte()]) {
                return Some(name.to_string());
            }
        }
    }
    
    if let Some(child) = node.child_by_field_name("name") {
        if let Ok(name) = std::str::from_utf8(&source[child.start_byte()..child.end_byte()]) {
            return Some(name.to_string());
        }
    }

    None
}

fn extract_signature(node: Node, source: &[u8]) -> String {
    let full_text = std::str::from_utf8(&source[node.start_byte()..node.end_byte()]).unwrap_or("");
    if let Some(line) = full_text.lines().next() {
        line.trim_end_matches('{').trim_end_matches(':').trim().to_string()
    } else {
        "".to_string()
    }
}

fn extract_docstring(node: Node, source: &[u8], lang: &str) -> Option<String> {
    if lang == "python" {
        if let Some(body) = node.child_by_field_name("body") {
            if let Some(expr) = body.child(0) {
                if expr.kind() == "expression_statement" {
                    if let Some(string) = expr.child(0) {
                        if string.kind() == "string" {
                            let text = std::str::from_utf8(&source[string.start_byte()..string.end_byte()]).unwrap_or("");
                            return Some(text.trim_matches('"').trim_matches('\'').to_string());
                        }
                    }
                }
            }
        }
    }
    None
}
