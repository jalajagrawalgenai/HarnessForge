"""Forge CLI — installed as the 'forge' command."""
import sys


def main():
    args = sys.argv[1:] if len(sys.argv) > 1 else ["serve"]

    if args[0] == "serve":
        port = 3000
        if len(args) > 1:
            try:
                port = int(args[1])
            except ValueError:
                print(f"Invalid port: {args[1]}")
                sys.exit(1)

        from forge_sdk import serve

        serve(port)
    elif args[0] == "version" or args[0] == "--version" or args[0] == "-V":
        from forge_sdk import get_version

        print(f"forge {get_version()}")
    elif args[0] == "help" or args[0] == "--help" or args[0] == "-h":
        print("Forge — AI Agent Harness")
        print()
        print("Usage: forge <command> [args]")
        print()
        print("Commands:")
        print("  serve [port]   Start the Forge dashboard server (default port 3000)")
        print("  version        Show version")
        print("  help           Show this help")
        print()
        print("Example:")
        print("  forge serve           # Start on port 3000 (or next available)")
        print("  forge serve 8080      # Start on port 8080")
    else:
        print(f"Unknown command: {args[0]}")
        print("Run 'forge help' for usage.")
        sys.exit(1)


if __name__ == "__main__":
    main()
