## Documentation References

Read through the `/docs/*.md` files to understand the project's design and UI layout.

## Tools

When running Bash tools in Yolo mode, always wrap long-running or network-dependent commands using timeout 30s <command>. If the command exits with status 124 (timeout), automatically log the failure, kill any remaining leaked processes, and retry the command exactly once before asking for help.

Abstain from broad filesystem searches - if file searching is required, search for specific files or directories instead of using broad wildcards or root directories. The only acceptable search locations are the project's source directory and any explicitly specified directories, eg cargo crate directories.
