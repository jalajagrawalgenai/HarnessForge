import * as vscode from 'vscode';
import { ForgeClient } from './client';

export class SessionsProvider implements vscode.TreeDataProvider<vscode.TreeItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    constructor(private client: ForgeClient) {}

    refresh() { this._onDidChangeTreeData.fire(); }

    async getChildren(): Promise<vscode.TreeItem[]> {
        const sessions = await this.client.listSessions();
        return sessions.map((s: any) => {
            const item = new vscode.TreeItem(
                `${s.id.slice(0, 8)} — ${s.status}`,
                vscode.TreeItemCollapsibleState.None
            );
            item.description = `Health: ${s.health_score?.toFixed(2) || 'N/A'} | ${s.interventions} interventions`;
            item.tooltip = `Task: ${s.task}\nAgent: ${s.agent_type}\nModel: ${s.model_id}`;
            item.iconPath = new vscode.ThemeIcon(
                s.health_score > 0.8 ? 'pass' : s.health_score > 0.5 ? 'warning' : 'error'
            );
            return item;
        });
    }

    getTreeItem(element: vscode.TreeItem): vscode.TreeItem {
        return element;
    }
}
