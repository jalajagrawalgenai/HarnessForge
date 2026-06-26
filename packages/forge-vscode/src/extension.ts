import * as vscode from 'vscode';
import { ForgeClient } from './client';
import { SessionsProvider } from './sessions';
import { HealthProvider } from './health';
import { InterventionsProvider } from './interventions';

let client: ForgeClient;

export function activate(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('forge');
    client = new ForgeClient(
        config.get<string>('apiEndpoint', 'http://localhost:9100'),
        config.get<string>('apiKey', '')
    );

    // Register sidebar views
    const sessionsProvider = new SessionsProvider(client);
    const healthProvider = new HealthProvider(client);
    const interventionsProvider = new InterventionsProvider(client);

    context.subscriptions.push(
        vscode.window.registerTreeDataProvider('forge.sessions', sessionsProvider),
        vscode.window.registerTreeDataProvider('forge.health', healthProvider),
        vscode.window.registerTreeDataProvider('forge.interventions', interventionsProvider)
    );

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('forge.runAgent', async () => {
            const task = await vscode.window.showInputBox({
                prompt: 'Enter the task for the agent',
                placeHolder: 'Write a function to validate email addresses'
            });
            if (task) {
                const result = await client.runAgent(task);
                vscode.window.showInformationMessage(
                    `Forge: Agent completed. Health: ${result.health_score?.toFixed(2) || 'N/A'}`
                );
            }
        }),

        vscode.commands.registerCommand('forge.replaySession', async () => {
            const sessionId = await vscode.window.showInputBox({
                prompt: 'Enter session ID to replay'
            });
            if (sessionId) {
                await client.replaySession(sessionId);
            }
        }),

        vscode.commands.registerCommand('forge.showHealth', () => {
            healthProvider.refresh();
            vscode.commands.executeCommand('forge.health.focus');
        }),

        vscode.commands.registerCommand('forge.configureHarness', async () => {
            const preset = await vscode.window.showQuickPick(
                ['solo', 'claude-code', 'langgraph', 'crewai', 'autogen', 'langchain',
                 'dspy', 'llamaindex', 'aider', 'cline', 'continue', 'copilot',
                 'cursor', 'windsurf', 'devin', 'custom'],
                { placeHolder: 'Select a harness preset' }
            );
            if (preset) {
                await vscode.workspace.getConfiguration('forge').update('preset', preset, true);
                vscode.window.showInformationMessage(`Forge: Preset set to "${preset}"`);
            }
        })
    );

    // Status bar: show active session health
    const healthStatus = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    healthStatus.text = '$(pulse) Forge';
    healthStatus.tooltip = 'Forge Agent Harness — click to view health';
    healthStatus.command = 'forge.showHealth';
    healthStatus.show();
    context.subscriptions.push(healthStatus);

    vscode.window.showInformationMessage('Forge Agent Harness activated. 12 observers, 16 detectors, 14 strategies ready.');
}

export function deactivate() {
    client?.dispose();
}
