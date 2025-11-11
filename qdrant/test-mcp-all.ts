#!/usr/bin/env bun
/**
 * Test all MCP Server tools
 */

import { spawn } from "child_process";

interface MCPRequest {
  jsonrpc: string;
  id: number;
  method: string;
  params?: any;
}

interface MCPResponse {
  jsonrpc: string;
  id: number;
  result?: any;
  error?: any;
}

async function sendMCPRequest(request: MCPRequest): Promise<MCPResponse> {
  return new Promise((resolve, reject) => {
    const mcp = spawn("bun", ["src/mcp-server.ts"], {
      stdio: ["pipe", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";

    mcp.stdout.on("data", (data) => {
      stdout += data.toString();
    });

    mcp.stderr.on("data", (data) => {
      stderr += data.toString();
    });

    mcp.on("close", (code) => {
      if (code !== 0) {
        reject(new Error(`MCP exited with code ${code}\n${stderr}`));
        return;
      }

      try {
        // Parse JSONRPC response
        const lines = stdout.trim().split("\n");
        const response = JSON.parse(lines[lines.length - 1]);
        resolve(response);
      } catch (error) {
        reject(new Error(`Failed to parse response: ${stdout}`));
      }
    });

    // Send request
    mcp.stdin.write(JSON.stringify(request) + "\n");
    mcp.stdin.end();
  });
}

async function testAllTools() {
  console.log("🧪 Testing All MCP Server Tools\n");

  let testId = 1;

  // Test 1: List Tools
  console.log("1️⃣ Testing list_tools...");
  try {
    const response = await sendMCPRequest({
      jsonrpc: "2.0",
      id: testId++,
      method: "tools/list",
    });
    const tools = response.result?.tools || [];
    console.log(`✅ Found ${tools.length} tools:`);
    tools.forEach((tool: any) => {
      console.log(`   - ${tool.name}: ${tool.description.substring(0, 50)}...`);
    });
    console.log();
  } catch (error) {
    console.error("❌ list_tools failed:", error);
  }

  // Test 2: vex_search
  console.log("2️⃣ Testing vex_search...");
  try {
    const response = await sendMCPRequest({
      jsonrpc: "2.0",
      id: testId++,
      method: "tools/call",
      params: {
        name: "vex_search",
        arguments: {
          query: "trait implementation",
          limit: 3,
        },
      },
    });
    console.log(
      "✅ vex_search result:",
      response.result?.content?.[0]?.text?.substring(0, 100)
    );
    console.log();
  } catch (error) {
    console.error("❌ vex_search failed:", error);
  }

  // Test 3: vex_find_examples
  console.log("3️⃣ Testing vex_find_examples...");
  try {
    const response = await sendMCPRequest({
      jsonrpc: "2.0",
      id: testId++,
      method: "tools/call",
      params: {
        name: "vex_find_examples",
        arguments: {
          feature: "generics",
          limit: 2,
        },
      },
    });
    console.log(
      "✅ vex_find_examples result:",
      response.result?.content?.[0]?.text?.substring(0, 100)
    );
    console.log();
  } catch (error) {
    console.error("❌ vex_find_examples failed:", error);
  }

  // Test 4: vex_find_todos
  console.log("4️⃣ Testing vex_find_todos...");
  try {
    const response = await sendMCPRequest({
      jsonrpc: "2.0",
      id: testId++,
      method: "tools/call",
      params: {
        name: "vex_find_todos",
        arguments: {
          priority: "high",
          limit: 5,
        },
      },
    });
    console.log(
      "✅ vex_find_todos result:",
      response.result?.content?.[0]?.text?.substring(0, 100)
    );
    console.log();
  } catch (error) {
    console.error("❌ vex_find_todos failed:", error);
  }

  // Test 5: vex_add_todo
  console.log("5️⃣ Testing vex_add_todo...");
  try {
    const response = await sendMCPRequest({
      jsonrpc: "2.0",
      id: testId++,
      method: "tools/call",
      params: {
        name: "vex_add_todo",
        arguments: {
          title: "MCP Test TODO",
          description: "This is a test TODO from MCP server",
          priority: "low",
        },
      },
    });
    console.log("✅ vex_add_todo result:", response.result?.content?.[0]?.text);
    console.log();
  } catch (error) {
    console.error("❌ vex_add_todo failed:", error);
  }

  // Test 6: vex_complete_todo
  console.log("6️⃣ Testing vex_complete_todo...");
  try {
    const response = await sendMCPRequest({
      jsonrpc: "2.0",
      id: testId++,
      method: "tools/call",
      params: {
        name: "vex_complete_todo",
        arguments: {
          title: "MCP Test TODO",
        },
      },
    });
    console.log(
      "✅ vex_complete_todo result:",
      response.result?.content?.[0]?.text
    );
    console.log();
  } catch (error) {
    console.error("❌ vex_complete_todo failed:", error);
  }

  // Test 7: vex_remove_todo
  console.log("7️⃣ Testing vex_remove_todo...");
  try {
    const response = await sendMCPRequest({
      jsonrpc: "2.0",
      id: testId++,
      method: "tools/call",
      params: {
        name: "vex_remove_todo",
        arguments: {
          title: "MCP Test TODO",
        },
      },
    });
    console.log(
      "✅ vex_remove_todo result:",
      response.result?.content?.[0]?.text
    );
    console.log();
  } catch (error) {
    console.error("❌ vex_remove_todo failed:", error);
  }

  console.log("🎉 All MCP tool tests completed!");
}

testAllTools().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
