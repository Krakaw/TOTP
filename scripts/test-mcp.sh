#!/bin/bash
# Helper to test MCP server

send_rpc() {
    local body="$1"
    local len=$(echo -n "$body" | wc -c | tr -d ' ')
    printf "Content-Length: %s\r\n\r\n%s" "$len" "$body"
}

{
    send_rpc '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
    send_rpc '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}'
    send_rpc '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
    send_rpc '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"trotp_generate","arguments":{"secret":"JBSWY3DPEHPK3PXP"}}}'
} | cargo run --bin trotp-mcp 2>&1
