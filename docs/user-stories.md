# User stories and entrypoints

## Goals and scope
- Context provides storage, search, and serving of project knowledge for both CLI and web consumers in a Rust workspace designed for multi-agent collaboration.
- The workspace components include the `context` CLI, the `context-web` server, and the React-based `web-ui` shell that together support CLI-first workflows with a future-facing web experience.

## User stories
| Journey | Story | CLI entrypoint | Web entrypoint |
| --- | --- | --- | --- |
| Capture knowledge | As a contributor, I want to store or update a document with tags so agents and teammates can reuse it later. | `context` CLI subcommands (e.g., `put`) defined in `context-cli/src/main.rs`. | (Future) UI actions will call the server; current shell renders via `web-ui/src/main.tsx`/`App.tsx`. |
| Retrieve and read | As a contributor, I want to fetch document content by key or ID in human-readable or JSON formats. | `context` CLI `get`/`cat` subcommands in `context-cli/src/main.rs`. | (Future) Document viewer to be wired to the Axum server scaffolding in `context-web/src/main.rs`. |
| Search and browse | As an analyst, I want to search documents, filter by project, and scan lists to find the right context quickly. | `find` and `ls` CLI subcommands in `context-cli/src/main.rs`. | Web UI search bar and project selector scaffolded in `web-ui/src/App.tsx`. |
| Cleanup and retention | As a maintainer, I want to remove stale items and garbage-collect tombstones to keep the store lean. | `rm` and `gc` CLI subcommands in `context-cli/src/main.rs`. | Future aging/cleanup views will ride on the `context-web` server once assets are served. |
| Web access | As a user, I want to launch a web interface to explore context without the CLI. | CLI includes `web`/`web-dev` wrappers in `context-cli/src/main.rs` to start the server. | Server entrypoint at `context-web/src/main.rs` and React bootstrapping at `web-ui/src/main.tsx`. |
| Agent guidance | As an agent integrator, I want Markdown documentation for agent usage. | `agent-doc` CLI subcommand in `context-cli/src/main.rs`. | `/agent-doc` endpoint in `context-web/src/main.rs` serves the same content. |

## Entry point references
- CLI: `context-cli/src/main.rs` defines the `context` binary and its subcommands for document lifecycle, web launchers, and agent tooling.
- Web server: `context-web/src/main.rs` hosts the Axum router that currently exposes health and agent-doc routes and will serve compiled UI assets.
- Web UI shell: `web-ui/src/main.tsx` mounts the React application that renders the current UI shell for search and project selection experiments.
