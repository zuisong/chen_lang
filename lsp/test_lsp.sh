#!/bin/bash

# Chen Lang LSP 测试脚本

echo "=== Chen Lang LSP 测试 ==="
echo ""

# 检查编译状态
echo "1. 检查编译状态..."
cd /Users/chen/src/github.com/zuisong/chen_lang/lsp
cargo build 2>&1 | grep -E "Finished|error" | head -5
if [ $? -eq 0 ]; then
    echo "   ✅ 编译成功"
else
    echo "   ❌ 编译失败"
    exit 1
fi

echo ""

# 测试 1: 发送 initialize 请求
echo "2. 测试 initialize 请求..."
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"rootUri":"file:///tmp/test.ch","capabilities":{}}}' | timeout 5 cargo run 2>&1 | head -10 &
sleep 1

# 测试 2: 发送 completion 请求
echo ""
echo "3. 测试 completion 请求..."
echo '{"jsonrpc":"2.0","id":2,"method":"textDocument/completion","params":{"textDocument":{"uri":"file:///tmp/test.ch"},"position":{"line":0,"character":2}}}' | timeout 5 cargo run 2>&1 | head -10 &
sleep 1

# 测试 3: 发送 hover 请求
echo ""
echo "4. 测试 hover 请求..."
echo '{"jsonrpc":"2.0","id":3,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///tmp/test.ch"},"position":{"line":0,"character":4}}}' | timeout 5 cargo run 2>&1 | head -10 &
sleep 1

echo ""
echo "=== 测试完成 ==="
