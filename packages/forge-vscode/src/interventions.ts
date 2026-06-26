import * as vscode from 'vscode';
import { ForgeClient } from './client';

export class InterventionsProvider implements vscode.TreeDataProvider<vscode.TreeItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    constructor(private client: ForgeClient) {}

    refresh() { this._onDidChangeTreeData.fire(); }

    async getChildren(): Promise<vscode.TreeItem[]> {
        // In production, fetch from API
        return [];
    }

    getTreeItem(element: vscode.TreeItem): vscode.TreeItem {
        return element;
    }
}
