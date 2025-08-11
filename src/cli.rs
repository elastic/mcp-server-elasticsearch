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

use crate::servers::elasticsearch;
use clap::Parser;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Elastic MCP server
#[derive(Debug, Parser)]
#[command(version)]
pub struct Cli {
    /// Container mode: change default http address, rewrite localhost to the host's address
    #[clap(global=true, long, env = "CONTAINER_MODE")]
    pub container_mode: bool,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Stdio(StdioCommand),
    Http(HttpCommand),
}

/// Start a streamable-HTTP server with optional SSE support
#[derive(Debug, Args)]
pub struct HttpCommand {
    /// Config file
    #[clap(short, long)]
    pub config: Option<PathBuf>,

    /// Address to listen to [default: 127.0.0.1:8080]
    #[clap(long, value_name = "IP_ADDRESS:PORT", env = "HTTP_ADDRESS")]
    pub address: Option<std::net::SocketAddr>,

    /// Also start an SSE server on '/sse'
    #[clap(long)]
    pub sse: bool,
}

/// Start an stdio server
#[derive(Debug, Args)]
pub struct StdioCommand {
    /// Config file
    #[clap(short, long)]
    pub config: Option<PathBuf>,
}

//---------------------------------------------------------------

// Reference material:
// https://modelcontextprotocol.io/quickstart/user
// https://code.visualstudio.com/docs/copilot/chat/mcp-servers
// https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-mcp-configuration.html
// https://github.com/landicefu/mcp-client-configuration-server

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stdio {
    /// Command to run (e.g. "npx", "docker")
    pub command: String,

    /// Command arguments
    pub args: Vec<String>,

    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Http {
    /// URL of the server
    pub url: String,

    /// HTTP headers to send with the request
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type")]
pub enum McpServer {
    //Builtin(BuiltinConfig),
    Sse(Http),
    StreamableHttp(Http),
    Stdio(Stdio),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub elasticsearch: elasticsearch::ElasticsearchMcpConfig,
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpServer>,
}
