## Documentation References

Read through the `/docs/*.md` files to understand the project's design and UI layout.

## Tools

When running Bash tools in Yolo mode, always wrap long-running or network-dependent commands using timeout 30s <command>. If the command exits with status 124 (timeout), automatically log the failure, kill any remaining leaked processes, and retry the command exactly once before asking for help.
