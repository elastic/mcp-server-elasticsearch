/*
 * Copyright Elasticsearch B.V. and contributors
 * SPDX-License-Identifier: Apache-2.0
 */

import { MCPInstrumentation } from '@arizeai/openinference-instrumentation-mcp'
import * as MCPServerStdioModule from '@modelcontextprotocol/sdk/server/stdio.js'

const mcpInstrumentation = new MCPInstrumentation()
mcpInstrumentation.manuallyInstrument({
  serverStdioModule: MCPServerStdioModule
})
