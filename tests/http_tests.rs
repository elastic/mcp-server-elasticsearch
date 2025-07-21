// Licensed to Elasticsearch B.V. under one or more contributor
// license agreements. See the NOTICE file distributed with
// this work for additional information regarding copyright
// ownership. Elasticsearch B.V. licenses this file to you under
// the Apache License, Version 2.0 (the "License"); you may
// not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use anyhow::bail;
use axum::Router;
use axum::extract::Path;
use elasticsearch_core_mcp_server::cli;
use futures_util::StreamExt;
use http::HeaderMap;
use http::header::{ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use rmcp::model::ToolAnnotations;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use sse_stream::SseStream;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};

/// Simple smoke test
#[tokio::test]
async fn http_tool_list() -> anyhow::Result<()> {
    let addr = find_address()?;

    let cli = cli::Cli {
        container_mode: false,
        command: cli::Command::Http(cli::HttpCommand {
            config: None,
            address: Some(addr),
            sse: false,
        }),
    };

    tokio::spawn(async move { cli.run().await });

    let url = format!("http://127.0.0.1:{}/mcp", addr.port());

    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    });

    let client = Client::builder().build()?;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json, text/event-stream")
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    let response_body: ListToolsResponse = parse_response(response).await?;

    let names = response_body
        .result
        .tools
        .iter()
        .map(|t| t.name.as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"search"));
    assert!(names.contains(&"list_indices"));
    assert!(names.contains(&"get_mappings"));
    Ok(())
}

// End-to-end test that spawns a mock ES server and calls the `list_indices` tool via http
#[tokio::test]
async fn end_to_end() -> anyhow::Result<()> {
    // Start an ES mock that will reply to list_indices
    let router = Router::new().route(
        "/_cat/indices/{index}",
        axum::routing::get(async move |headers: HeaderMap, Path(index): Path<String>| {
            // Check parameter forwarding
            assert_eq!(index, "test-index");
            // Check API key
            assert_eq!(
                headers.get("Authorization").unwrap().to_str().unwrap(),
                "ApiKey value-from-the-test"
            );
            axum::Json(json!([
              {
                "index": "test-index",
                "status": "open",
                "docs.count": "100"
              }
            ]))
        }),
    );

    let listener = tokio::net::TcpListener::bind(LOCALHOST_0).await?;

    // SAFETY: works since this is the only test in this module that sets env vars
    // TODO: refactor the CLI to accept an alternate source of key/values
    unsafe {
        std::env::set_var("ES_URL", format!("http://127.0.0.1:{}/", listener.local_addr()?.port()));
    }
    let server = axum::serve(listener, router);
    tokio::spawn(async { server.await });

    // Start an http MCP server
    let addr = find_address()?;
    let cli = cli::Cli {
        container_mode: false,
        command: cli::Command::Http(cli::HttpCommand {
            config: None,
            address: Some(addr),
            sse: false,
        }),
    };

    tokio::spawn(async move { cli.run().await });
    let url = format!("http://127.0.0.1:{}/mcp", addr.port());
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "list_indices",
            "arguments": {
                "index_pattern": "test-index"
            }
        }
    });

    let client = Client::builder().build()?;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json, text/event-stream")
        .header("Authorization", "ApiKey value-from-the-test")
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    let response_body: serde_json::Value = parse_response(response).await?;

    assert_eq!(response_body["result"]["content"][0]["text"], "Found 1 indices:");
    assert_eq!(
        response_body["result"]["content"][1]["text"],
        "[{\"index\":\"test-index\",\"status\":\"open\",\"docs.count\":100}]"
    );

    Ok(())
}

const LOCALHOST_0: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);

fn find_address() -> anyhow::Result<SocketAddr> {
    // Find a free port
    Ok(TcpListener::bind(LOCALHOST_0)?.local_addr()?)
}

async fn parse_response<T: DeserializeOwned>(response: reqwest::Response) -> anyhow::Result<T> {
    let result = match response.headers().get(CONTENT_TYPE) {
        Some(v) if v == "application/json" => response.json().await?,
        Some(v) if v == "text/event-stream" => {
            let mut stream = SseStream::from_byte_stream(response.bytes_stream());
            match stream.next().await {
                None => bail!("No data"),
                Some(Err(e)) => bail!("Bad SSE stream: {}", e),
                Some(Ok(sse)) => {
                    let data = sse.data.unwrap();
                    serde_json::from_str(&data)?
                }
            }
        }
        _ => {
            panic!("Unexpected content type");
        }
    };

    Ok(result)
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ListToolsResponse {
    jsonrpc: String,
    id: i64,
    result: ToolResult,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ToolResult {
    tools: Vec<Tool>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Tool {
    name: String,
    description: String,
    input_schema: Option<serde_json::Value>,
    annotations: Option<ToolAnnotations>,
}
