//! `ca72-mcp`: the CA-72's Model Context Protocol server for AI agents, which an MCP client
//! starts and talks to on standard input and output (`ca72_plugin::agent::bridge`).

fn main() {
    std::process::exit(ca72_plugin::agent::bridge::main());
}
