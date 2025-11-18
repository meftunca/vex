// Vex Language Client Extension
// Activates and manages the LSP connection

import * as path from "path";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

function createLanguageClient(): LanguageClient {
  // LSP server binary path - use development binary
  const serverModule = path.join(
    process.env.HOME || "",
    ".cargo",
    "target",
    "debug",
    "vex-lsp"
  );

  // Server options: run the Rust LSP binary
  const serverOptions: ServerOptions = {
    run: {
      command: serverModule,
      transport: TransportKind.stdio,
    },
    debug: {
      command: serverModule,
      transport: TransportKind.stdio,
      options: {
        env: {
          ...process.env,
          RUST_LOG: "info",
          RUST_BACKTRACE: "1",
        },
      },
    },
  };

  // Client options: when to activate
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "vex" }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.{vx,vxc}"),
    },
    outputChannelName: "Vex Language Server",
  };

  return new LanguageClient(
    "vexLanguageServer",
    "Vex Language Server",
    serverOptions,
    clientOptions
  );
}

export function activate(context: vscode.ExtensionContext) {
  console.log("Vex Language Extension activating...");
  // Create and start the client (also starts the server)
  client = createLanguageClient();
  client.start().catch((e) => {
    console.error("Failed to start Vex LSP server:", e);
    vscode.window.showErrorMessage(
      "Failed to start Vex Language Server: " + (e?.message ?? e)
    );
  });

  console.log("Vex Language Extension activated");

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand("vex.restartServer", async () => {
      try {
        if (client) {
          console.log("Attempting to stop the Vex LSP server...");
          await client.stop();
          console.log("Stopped the Vex LSP server");
        }

        // Recreate a fresh client and start
        client = createLanguageClient();
        console.log("Starting new Vex LSP server instance...");
        await client.start();
        vscode.window.showInformationMessage("Vex LSP server restarted");
      } catch (err) {
        console.error("Failed to restart Vex LSP server:", err);
        vscode.window.showErrorMessage(
          "Failed to restart Vex LSP server: " + (err as Error).message
        );
      }
    })
  );
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  try {
    return client.stop();
  } catch (err) {
    console.error("Error stopping client on deactivate:", err);
    return undefined;
  }
}

// This function is no longer needed as we rely on the PATH.
/*
function findServerBinary(): string | null {
  const possiblePaths = [
    // Development: ~/.cargo/target/debug/vex-lsp
    path.join(process.env.HOME || "", ".cargo", "target", "debug", "vex-lsp"),
    // Release: ~/.cargo/target/release/vex-lsp
    path.join(process.env.HOME || "", ".cargo", "target", "release", "vex-lsp"),
    // Workspace: vex_lang/target/debug/vex-lsp
    path.join(
      vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || "",
      "target",
      "debug",
      "vex-lsp"
    ),
    // Workspace: vex_lang/target/release/vex-lsp
    path.join(
      vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || "",
      "target",
      "release",
      "vex-lsp"
    ),
  ];

  for (const p of possiblePaths) {
    if (require("fs").existsSync(p)) {
      return p;
    }
  }
  return null;
}
*/
