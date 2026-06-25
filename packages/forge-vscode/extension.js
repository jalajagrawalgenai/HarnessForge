const vscode = require('vscode');
function activate(context) {
    context.subscriptions.push(
        vscode.commands.registerCommand('forge.runAgent', () => vscode.window.showInputBox({prompt:'Task'}).then(t => t && vscode.window.showInformationMessage(`Forge: Running "${t}"`))),
        vscode.commands.registerCommand('forge.replaySession', () => vscode.window.showInformationMessage('Forge: Opening replay')),
        vscode.commands.registerCommand('forge.explainSession', () => vscode.window.showInformationMessage('Forge: Generating report'))
    );
}
module.exports = { activate };
