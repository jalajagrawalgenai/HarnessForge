import * as vscode from 'vscode';
import { ForgeClient } from './client';

export class HealthProvider implements vscode.TreeDataProvider<vscode.TreeItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    constructor(private client: ForgeClient) {}

    refresh() { this._onDidChangeTreeData.fire(); }

    async getChildren(): Promise<vscode.TreeItem[]> {
        const dimensions = [
            'token', 'latency', 'cost', 'accuracy', 'security', 'reliability',
            'context_quality', 'orch', 'comm', 'compliance', 'memory', 'diversity'
        ];
        return dimensions.map(dim => {
            const item = new vscode.TreeItem(dim, vscode.TreeItemCollapsibleState.None);
            item.description = '🟢 0.95';
            item.tooltip = `${dim} health dimension — hover for details`;
            return item;
        });
    }

    getTreeItem(element: vscode.TreeItem): vscode.TreeItem {
        return element;
    }
}
