use anyhow::Result;
use rust_mcp_sdk::{McpServer, mcp_tool, tool_box};
use rust_mcp_sdk::mcp_server::ServerHandler;
use rust_mcp_sdk::error::SdkResult;
use rust_mcp_sdk::schema::{
    CallToolRequestParams, CallToolResult, ListToolsResult, Content, RpcError, PaginatedRequestParams,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use crate::storage::Storage;
use crate::search::Searcher;
use crate::indexer::Indexer;
use async_trait::async_trait;

#[derive(Debug, Deserialize, JsonSchema)]
#[mcp_tool(name = "index_folder", description = "Index a local folder and return indexing statistics.")]
pub struct IndexFolderArgs {
    pub path: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[mcp_tool(name = "search_symbols", description = "Find functions/classes by name. ~200 tokens per lookup.")]
pub struct SearchSymbolsArgs {
    pub query: String,
    pub repo: Option<String>,
    pub kind: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[mcp_tool(name = "get_symbol", description = "Get full source of a symbol by byte offsets.")]
pub struct GetSymbolArgs {
    pub repo: String,
    pub file: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[mcp_tool(name = "semantic_search", description = "Find symbols by meaning/natural language.")]
pub struct SemanticSearchArgs {
    pub query: String,
    pub repo: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[mcp_tool(name = "search_routes", description = "Find HTTP routes indexed from a repo.")]
pub struct SearchRoutesArgs {
    pub repo: String,
    pub method: Option<String>,
    pub path_contains: Option<String>,
    pub limit: Option<usize>,
}

tool_box!(RustDexToolbox, [
    IndexFolderArgs,
    SearchSymbolsArgs,
    GetSymbolArgs,
    SemanticSearchArgs,
    SearchRoutesArgs
]);

pub struct RustDexHandler {
    pub storage: Storage,
}

#[async_trait]
impl ServerHandler for RustDexHandler {
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> SdkResult<ListToolsResult> {
        Ok(ListToolsResult {
            tools: RustDexToolbox::tools(),
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> SdkResult<CallToolResult> {
        let tool_call = RustDexToolbox::try_from(params).map_err(|e| RpcError::invalid_params(e.to_string()))?;
        
        match tool_call {
            RustDexToolbox::IndexFolderArgs(args) => {
                let mut indexer = Indexer::new(self.storage.clone())
                    .map_err(|e| RpcError::internal_error(e.to_string()))?;
                let info = indexer.index_folder(std::path::Path::new(&args.path), args.name)
                    .map_err(|e| RpcError::internal_error(e.to_string()))?;
                
                Ok(CallToolResult::new(vec![
                    Content::text(format!("Indexed repo: {} at {}", info.name, info.db_path.display()))
                ]))
            }
            RustDexToolbox::SearchSymbolsArgs(args) => {
                let searcher = Searcher::new(self.storage.clone())
                    .map_err(|e| RpcError::internal_error(e.to_string()))?;
                let symbols = searcher.search_symbols(&args.query, args.repo.as_deref(), args.kind.as_deref(), args.limit.unwrap_or(20))
                    .map_err(|e| RpcError::internal_error(e.to_string()))?;
                
                let json = serde_json::to_string(&symbols).unwrap();
                Ok(CallToolResult::new(vec![Content::text(json)]))
            }
            RustDexToolbox::GetSymbolArgs(args) => {
                let searcher = Searcher::new(self.storage.clone())
                    .map_err(|e| RpcError::internal_error(e.to_string()))?;
                let source = searcher.get_symbol_source(&args.repo, &args.file, args.start_byte, args.end_byte)
                    .map_err(|e| RpcError::internal_error(e.to_string()))?;
                
                Ok(CallToolResult::new(vec![Content::text(source)]))
            }
            RustDexToolbox::SemanticSearchArgs(args) => {
                let mut searcher = Searcher::new(self.storage.clone())
                    .map_err(|e| RpcError::internal_error(e.to_string()))?;
                let results = searcher.search_semantic(&args.query, args.repo.as_deref(), args.limit.unwrap_or(10))
                    .map_err(|e| RpcError::internal_error(e.to_string()))?;
                
                let mut output = String::new();
                for (sym, score) in results {
                    output.push_str(&format!("[{:.4}] {} [{}] in {}\n", score, sym.name, sym.kind, sym.file));
                }
                Ok(CallToolResult::new(vec![Content::text(output)]))
            }
            RustDexToolbox::SearchRoutesArgs(args) => {
                let searcher = Searcher::new(self.storage.clone())
                    .map_err(|e| RpcError::internal_error(e.to_string()))?;
                let routes = searcher.search_routes(&args.repo, args.method.as_deref(), args.path_contains.as_deref(), args.limit.unwrap_or(50))
                    .map_err(|e| RpcError::internal_error(e.to_string()))?;
                
                let json = serde_json::to_string(&routes).unwrap();
                Ok(CallToolResult::new(vec![Content::text(json)]))
            }
        }
    }
}
