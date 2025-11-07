// Vex Language Client Extension
// Activates and manages the LSP connection

import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
  console.log("Vex Language Extension activating...");

  // LSP server binary path
  // IMPORTANT: The server binary must be on the system's PATH.
  const serverModule = "vex-lsp";

  console.log(`Found Vex LSP server at: ${serverModule}`);

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
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.vx"),
    },
    outputChannelName: "Vex Language Server",
  };

  // Create and start the client
  client = new LanguageClient(
    "vexLanguageServer",
    "Vex Language Server",
    serverOptions,
    clientOptions
  );

  // Start the client (also starts the server)
  client.start();

  console.log("Vex Language Extension activated");

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand("vex.restartServer", async () => {
      await client.stop();
      await client.start();
      vscode.window.showInformationMessage("Vex LSP server restarted");
    })
  );
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
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
