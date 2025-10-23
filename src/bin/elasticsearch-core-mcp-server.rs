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

use clap::Parser;
use elasticsearch_core_mcp_server::cli::Cli;
use std::io::ErrorKind;
use tracing_subscriber::EnvFilter;
// To test with stdio, use npx @modelcontextprotocol/inspector cargo run -p elastic-mcp

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Also accept .env files
    match dotenvy::dotenv() {
        Err(dotenvy::Error::Io(io_err)) if io_err.kind() == ErrorKind::NotFound => {}
        Err(err) => return Err(err)?,
        Ok(_) => {}
    }

    let env_args = std::env::vars().find(|(k, _v)| k == "CLI_ARGS").map(|(_k, v)| v);

    let cli = if let Some(env_args) = env_args {
        // Concatenate arg[0] with the ARGS value split on whitespaces
        // Note: we don't handle shell-style string quoting and character escaping
        let arg0 = std::env::args().next().unwrap();
        let mut args = vec![arg0.as_str()];
        args.extend(env_args.split_whitespace());

        Cli::parse_from(args)
    } else {
        Cli::parse()
    };

    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Elasticsearch MCP server, version {}", env!("CARGO_PKG_VERSION"));

    cli.run().await
}
