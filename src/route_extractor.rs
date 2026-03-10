use regex::bytes::Regex;
use crate::storage::RouteInfo;

pub fn extract_routes(source: &[u8], lang: &str, repo: &str, file: &str) -> Vec<RouteInfo> {
    let mut results = Vec::new();
    let lang = lang.to_lowercase();

    if lang == "python" {
        // @app.route("/path", methods=["GET", "POST"])
        let py_route = Regex::new(
            r#"(?s)@\w+\.route\(\s*["']([^"']+)["']\s*(?:,\s*methods\s*=\s*\[([^\]]*)\])?\s*\)\s*\n\s*(?:async\s+)?def\s+(\w+)"#
        ).unwrap();

        for m in py_route.captures_iter(source) {
            let path = String::from_utf8_lossy(&m[1]).to_string();
            let raw_methods = m.get(2).map(|x| x.as_bytes()).unwrap_or(b"");
            let methods = if !raw_methods.is_empty() {
                parse_methods(raw_methods)
            } else {
                vec!["GET".to_string()]
            };
            let handler = String::from_utf8_lossy(&m[3]).to_string();
            
            for method in methods {
                results.push(RouteInfo {
                    id: None,
                    repo: repo.to_string(),
                    file: file.to_string(),
                    method,
                    path: path.clone(),
                    handler: Some(handler.clone()),
                    start_byte: m.get(0).unwrap().start(),
                    end_byte: m.get(0).unwrap().end(),
                });
            }
        }

        // @app.get("/path")
        let py_shorthand = Regex::new(
            r#"(?s)@\w+\.(get|post|put|delete|patch|head|options)\(\s*["']([^"']+)["']\s*\)\s*\n\s*(?:async\s+)?def\s+(\w+)"#
        ).unwrap();

        for m in py_shorthand.captures_iter(source) {
            let method = String::from_utf8_lossy(&m[1]).to_string().to_uppercase();
            let path = String::from_utf8_lossy(&m[2]).to_string();
            let handler = String::from_utf8_lossy(&m[3]).to_string();
            
            results.push(RouteInfo {
                id: None,
                repo: repo.to_string(),
                file: file.to_string(),
                method,
                path,
                handler: Some(handler),
                start_byte: m.get(0).unwrap().start(),
                end_byte: m.get(0).unwrap().end(),
            });
        }
    } else if lang == "javascript" || lang == "typescript" {
        // app.get("/path", handler)
        let js_method = Regex::new(
            r#"\b\w+\.(get|post|put|delete|patch|head|options|all)\(\s*["'`]([^"'`]+)["'`]\s*,\s*(\w+)"#
        ).unwrap();

        for m in js_method.captures_iter(source) {
            let method = String::from_utf8_lossy(&m[1]).to_string().to_uppercase();
            let path = String::from_utf8_lossy(&m[2]).to_string();
            let handler = String::from_utf8_lossy(&m[3]).to_string();
            
            results.push(RouteInfo {
                id: None,
                repo: repo.to_string(),
                file: file.to_string(),
                method,
                path,
                handler: Some(handler),
                start_byte: m.get(0).unwrap().start(),
                end_byte: m.get(0).unwrap().end(),
            });
        }
    }

    results
}

fn parse_methods(raw: &[u8]) -> Vec<String> {
    raw.split(|&b| b == b',')
        .map(|m| {
            let s = String::from_utf8_lossy(m).to_string();
            s.trim_matches(|c| c == '\"' || c == '\'' || c == ' ').to_uppercase()
        })
        .filter(|m| !m.is_empty())
        .collect()
}
