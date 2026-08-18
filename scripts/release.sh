#!/bin/bash
# release.sh — 发版脚本:自动升级版本号 + 打 tag + 推送触发 CI 构建发布
#
# 用法:
#   ./scripts/release.sh                 # 自动递增 patch 版本(4.0.1 -> 4.0.2)
#   ./scripts/release.sh 4.1.0           # 指定版本号
#
# 流程:检查工作区 -> 更新版本号 -> 提交 -> 打 tag -> push tag(触发自动构建发布)

set -e

# ---------- 配置 ----------
REMOTE=origin
BRANCH=master
PKG=package.json
CONF=src-tauri/tauri.conf.json
CARGO=src-tauri/Cargo.toml

# ---------- 1. 检查工作区 ----------
if [ -n "$(git status --porcelain)" ]; then
    echo "❌ 工作区有未提交的改动,请先 commit 或 stash"
    git status --short
    exit 1
fi

# ---------- 2. 确定版本号 ----------
CURRENT_VERSION=$(grep '"version"' "$PKG" | head -1 | sed 's/.*: *"\([0-9.]*\)".*/\1/')
echo "当前版本: $CURRENT_VERSION"

if [ -z "$1" ]; then
    # 自动递增 patch:4.0.1 -> 4.0.2
    NEW_VERSION=$(echo "$CURRENT_VERSION" | awk -F. '{$3=$3+1; printf "%d.%d.%d", $1, $2, $3}')
else
    NEW_VERSION=$1
fi
echo "新版本: $NEW_VERSION"

# ---------- 3. 校验版本号格式 ----------
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "❌ 版本号格式错误: $NEW_VERSION (应为 x.y.z)"
    exit 1
fi

# 检查 tag 是否已存在
if git rev-parse "refs/tags/$NEW_VERSION" >/dev/null 2>&1; then
    echo "❌ tag $NEW_VERSION 已存在,请换一个版本号"
    exit 1
fi

# ---------- 4. 更新三处版本号 ----------
echo "更新版本号..."
sed -i "s/\"version\": *\"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" "$PKG"
sed -i "s/\"version\": *\"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" "$CONF"
sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO"

echo "验证更新:"
grep '"version"' "$PKG" | head -1
grep '"version"' "$CONF" | head -1
grep '^version' "$CARGO" | head -1

# ---------- 5. 提交版本号改动 ----------
git add "$PKG" "$CONF" "$CARGO"
git commit -m "chore: bump version to $NEW_VERSION"

# ---------- 6. 打 tag ----------
git tag "$NEW_VERSION"

# ---------- 7. 推送(触发 CI 自动构建发布) ----------
echo "推送 tag $NEW_VERSION 触发 CI 构建发布..."
git push "$REMOTE" "$BRANCH"
git push "$REMOTE" "refs/tags/$NEW_VERSION"

echo ""
echo "✅ 发版完成:"
echo "  版本: $NEW_VERSION"
echo "  tag: $NEW_VERSION"
echo "  CI 已触发,会自动构建并发布所有平台安装包 + update.json"
echo "  查看进度: https://github.com/luyanfeng/newpot/actions"
