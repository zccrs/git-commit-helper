#!/bin/bash
set -e

# 编译项目
cargo build --release

# 安装到当前 git 仓库
./target/release/git-commit-helper install

# 运行配置向导
./target/release/git-commit-helper config

echo "安装完成！"
