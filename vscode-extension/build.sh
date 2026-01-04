#!/bin/bash

set -e

echo "🚀 构建 Chen Lang VS Code 扩展..."
echo ""

# 检查 Bun
if ! command -v bun &> /dev/null; then
    echo "❌ 错误: 未找到 Bun"
    echo "请先安装 Bun: https://bun.sh/"
    exit 1
fi

echo "✅ Bun 版本: $(bun --version)"
echo ""

# 进入扩展目录
cd "$(dirname "$0")"

# 安装依赖
echo "📦 安装依赖..."
bun install

# 编译
echo "🔨 编译 TypeScript..."
bun run compile

# 打包
echo "📦 打包 VSIX..."
bun run package

echo ""
echo "✅ 构建完成！"
echo ""
echo "生成的文件: chen-lang-0.2.0.vsix"
echo ""
echo "安装方法:"
echo "  code --install-extension chen-lang-0.2.0.vsix"
echo ""
echo "或在 VS Code 中:"
echo "  Cmd+Shift+P > Extensions: Install from VSIX..."
echo ""
