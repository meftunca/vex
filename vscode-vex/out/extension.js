"use strict";
// Vex Language Client Extension
// Activates and manages the LSP connection
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
let client;
function createLanguageClient() {
    // LSP server binary path
    const serverModule = "vex-lsp";
    // Server options: run the Rust LSP binary
    const serverOptions = {
        run: {
            command: serverModule,
            transport: node_1.TransportKind.stdio,
        },
        debug: {
            command: serverModule,
            transport: node_1.TransportKind.stdio,
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
    const clientOptions = {
        documentSelector: [{ scheme: "file", language: "vex" }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher("**/*.{vx,vxc}"),
        },
        outputChannelName: "Vex Language Server",
    };
    return new node_1.LanguageClient("vexLanguageServer", "Vex Language Server", serverOptions, clientOptions);
}
function activate(context) {
    console.log("Vex Language Extension activating...");
    // Create and start the client (also starts the server)
    client = createLanguageClient();
    client.start().catch((e) => {
        console.error("Failed to start Vex LSP server:", e);
        vscode.window.showErrorMessage("Failed to start Vex Language Server: " + (e?.message ?? e));
    });
    console.log("Vex Language Extension activated");
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand("vex.restartServer", async () => {
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
        }
        catch (err) {
            console.error("Failed to restart Vex LSP server:", err);
            vscode.window.showErrorMessage("Failed to restart Vex LSP server: " + err.message);
        }
    }));
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    try {
        return client.stop();
    }
    catch (err) {
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
//# sourceMappingURL=extension.js.map