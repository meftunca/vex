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
const path = __importStar(require("path"));
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    console.log('Vex Language Extension activating...');
    // LSP server binary path
    const serverModule = findServerBinary();
    if (!serverModule) {
        vscode.window.showErrorMessage('Vex LSP server not found. Please build with: cargo build --release -p vex-lsp');
        return;
    }
    console.log(`Found Vex LSP server at: ${serverModule}`);
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
                    RUST_LOG: 'info',
                    RUST_BACKTRACE: '1',
                },
            },
        },
    };
    // Client options: when to activate
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'vex' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.vx'),
        },
        outputChannelName: 'Vex Language Server',
    };
    // Create and start the client
    client = new node_1.LanguageClient('vexLanguageServer', 'Vex Language Server', serverOptions, clientOptions);
    // Start the client (also starts the server)
    client.start();
    console.log('Vex Language Extension activated');
    // Register commands
    context.subscriptions.push(vscode.commands.registerCommand('vex.restartServer', async () => {
        await client.stop();
        await client.start();
        vscode.window.showInformationMessage('Vex LSP server restarted');
    }));
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
// Find the LSP server binary
function findServerBinary() {
    const possiblePaths = [
        // Development: ~/.cargo/target/debug/vex-lsp
        path.join(process.env.HOME || '', '.cargo', 'target', 'debug', 'vex-lsp'),
        // Release: ~/.cargo/target/release/vex-lsp
        path.join(process.env.HOME || '', '.cargo', 'target', 'release', 'vex-lsp'),
        // Workspace: vex_lang/target/debug/vex-lsp
        path.join(vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '', 'target', 'debug', 'vex-lsp'),
        // Workspace release: vex_lang/target/release/vex-lsp
        path.join(vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '', 'target', 'release', 'vex-lsp'),
    ];
    for (const p of possiblePaths) {
        try {
            const fs = require('fs');
            if (fs.existsSync(p)) {
                return p;
            }
        }
        catch (e) {
            // Ignore
        }
    }
    return null;
}
//# sourceMappingURL=extension.js.map