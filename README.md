# Git Commit Helper

一个自动翻译 git commit message 的工具。

## 环境要求

- Rust 开发环境 (推荐使用 [rustup](https://rustup.rs/) 安装)
- Git
- 支持的操作系统：Linux、macOS、Windows

## 安装

### 方式一：使用安装脚本（推荐，仅支持 Linux/macOS）

```bash
# 给安装脚本添加执行权限
chmod +x install.sh

# 运行安装脚本
./install.sh
```

### 方式二：手动安装

1. 确保已安装 Rust 环境
```bash
# 检查 Rust 是否安装
rustc --version
cargo --version

# 如果未安装，使用 rustup 安装
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. 编译项目
```bash
# 开发版本编译（编译速度快，文件较大）
cargo build

# 发布版本编译（编译较慢，文件更小、运行更快）
cargo build --release
```

3. 安装到指定的 git 仓库
```bash
# 开发版本
./target/debug/git-commit-helper install

# 发布版本
./target/release/git-commit-helper install
```

4. 配置 AI 服务
```bash
# 开发版本
./target/debug/git-commit-helper config

# 发布版本
./target/release/git-commit-helper config
```

## 调试与开发

如果您需要调试或开发新功能：

```bash
# 运行测试
cargo test

# 检查代码格式
cargo fmt

# 运行静态分析
cargo clippy

# 构建文档
cargo doc --open
```

## 使用方法

安装完成后，直接使用 git commit 即可，工具会自动检测并询问是否需要翻译提交信息。

## 配置文件位置

- Linux: ~/.config/git-commit-helper/config.json
- macOS: ~/Library/Application Support/com.githelper.git-commit-helper/config.json
- Windows: %APPDATA%\githelper\git-commit-helper\config.json

## 命令行工具使用

```bash
# 列出所有配置的AI服务
git-commit-helper list

# 测试指定编号的AI服务
git-commit-helper test --index <编号>

# 测试指定编号的AI服务，使用自定义文本
git-commit-helper test --index <编号> --text "要翻译的文本"
```
