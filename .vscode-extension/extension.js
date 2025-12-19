const vscode = require('vscode');
const path = require('path');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let client;
let outputChannel;

function activate(context) {
    // Create output channel for extension debugging
    outputChannel = vscode.window.createOutputChannel('loft LSP Debug');
    outputChannel.appendLine('=== Loft Language Server Extension Activated ===');
    console.log('loft Language Server extension is now active!');

    // Register restart command
    const restartCommand = vscode.commands.registerCommand('loft.restartLanguageServer', async () => {
        outputChannel.appendLine('Restart command triggered');
        await restartLanguageServer();
    });
    context.subscriptions.push(restartCommand);
    outputChannel.appendLine('Registered restart command: twang.restartLanguageServer');

    // Start the language server
    startLanguageServer(context);
}

function startLanguageServer(context) {
    outputChannel.appendLine('Starting language server...');

    // Find the loft-lsp binary
    const serverCommand = findLoftLsp();
    
    if (!serverCommand) {
        outputChannel.appendLine('ERROR: loft-lsp binary not found');
        vscode.window.showWarningMessage(
            'loft LSP binary not found. Language server features will be disabled. ' +
            'Please build the loft-lsp binary with: cargo build --bin loft-lsp'
        );
        return;
    }

    outputChannel.appendLine(`LSP server binary found: ${serverCommand}`);

    const serverOptions = {
        command: serverCommand,
        transport: TransportKind.stdio
    };

    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'loft' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.lf')
        },
        // Explicitly enable all supported LSP features
        initializationOptions: {},
        // Enable diagnostics, hover, and completion
        diagnosticCollectionName: 'loft',
        // Output channel for debugging
        outputChannelName: 'loft Language Server',
        // Trace server communication for debugging
        traceOutputChannel: vscode.window.createOutputChannel('loft LSP Trace')
    };

    outputChannel.appendLine('Creating LSP client...');
    client = new LanguageClient(
        'loftLanguageServer',
        'loft Language Server',
        serverOptions,
        clientOptions
    );

    // Log client lifecycle events
    client.onDidChangeState((event) => {
        outputChannel.appendLine(`LSP client state changed: ${JSON.stringify(event)}`);
    });

    outputChannel.appendLine('Starting LSP client...');
    client.start().then(() => {
        outputChannel.appendLine('LSP client started successfully');
    }).catch((error) => {
        outputChannel.appendLine(`ERROR starting LSP client: ${error}`);
    });

    context.subscriptions.push(client);
}

async function restartLanguageServer() {
    outputChannel.appendLine('Restarting language server...');
    
    if (!client) {
        outputChannel.appendLine('No active client to restart');
        return;
    }

    try {
        outputChannel.appendLine('Stopping current client...');
        await client.stop();
        outputChannel.appendLine('Client stopped successfully');
        
        // Give it a moment before restarting
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        outputChannel.appendLine('Starting new client...');
        await client.start();
        outputChannel.appendLine('Client restarted successfully');
    } catch (error) {
        outputChannel.appendLine(`ERROR during restart: ${error}`);
        vscode.window.showErrorMessage(`Failed to restart Twang Language Server: ${error.message}`);
    }
}

function findLoftLsp() {
    const { execSync } = require('child_process');
    
    outputChannel.appendLine('Searching for loft-lsp binary...');
    
    // Try to find loft-lsp in PATH
    try {
        const result = execSync('which loft-lsp', { encoding: 'utf8' }).trim();
        if (result) {
            outputChannel.appendLine(`Found in PATH: ${result}`);
            return result;
        }
    } catch (e) {
        outputChannel.appendLine('Not found in PATH, checking workspace locations...');
    }

    // Try in the workspace
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (workspaceFolders) {
        for (const folder of workspaceFolders) {
            const possiblePaths = [
                path.join(folder.uri.fsPath, 'target', 'debug', 'loft-lsp'),
                path.join(folder.uri.fsPath, 'target', 'release', 'loft-lsp'),
            ];
            
            outputChannel.appendLine(`Checking workspace: ${folder.uri.fsPath}`);
            for (const p of possiblePaths) {
                outputChannel.appendLine(`  Trying: ${p}`);
                const fs = require('fs');
                if (fs.existsSync(p)) {
                    outputChannel.appendLine(`  Found: ${p}`);
                    return p;
                }
            }
        }
    }

    outputChannel.appendLine('loft-lsp binary not found');
    return null;
}

function deactivate() {
    outputChannel.appendLine('Deactivating extension...');
    if (!client) {
        outputChannel.appendLine('No active client to stop');
        return undefined;
    }
    outputChannel.appendLine('Stopping LSP client...');
    return client.stop().then(() => {
        outputChannel.appendLine('LSP client stopped successfully');
    });
}

module.exports = {
    activate,
    deactivate
};
