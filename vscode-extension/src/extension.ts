import * as path from 'path';
import { workspace, ExtensionContext, window } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export async function activate(context: ExtensionContext) {
    console.log('Chen Lang extension is now active!');

    try {
        // 获取 LSP 服务器路径配置
        const config = workspace.getConfiguration('chenLang');
        const lspPath = config.get<string>('lsp.path', 'chen_lang_lsp');

        console.log(`LSP server path: ${lspPath}`);

        // LSP 服务器配置
        const serverOptions: ServerOptions = {
            command: lspPath,
            args: [],
            transport: TransportKind.stdio
        };

        // 客户端配置
        const clientOptions: LanguageClientOptions = {
            documentSelector: [{ scheme: 'file', language: 'chen' }],
            synchronize: {
                fileEvents: workspace.createFileSystemWatcher('**/*.ch')
            }
        };

        // 创建并启动语言客户端
        client = new LanguageClient(
            'chenLangLsp',
            'Chen Lang Language Server',
            serverOptions,
            clientOptions
        );

        // 启动客户端，这也会启动服务器
        console.log('Starting Chen Lang LSP client...');
        await client.start();
        console.log('Chen Lang LSP client started successfully!');

    } catch (error) {
        console.error('Failed to start Chen Lang LSP:', error);
        window.showErrorMessage(`Chen Lang LSP failed to start: ${error}`);
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
