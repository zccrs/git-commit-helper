# git-commit-helper

一个帮助生成和翻译 Git 提交信息的命令行工具，支持多个 AI 服务，可以进行中英互译。

## 功能特点

- 支持多个 AI 服务：DeepSeek、OpenAI、Claude、Github Copilot、Gemini、Grok
- 支持自定义 API 地址和模型
- 支持中英双语提交信息
- 支持自动检测中文内容并询问是否翻译
- 支持根据暂存区内容自动生成提交信息

## 安装

```bash
# 克隆仓库
git clone https://github.com/zccrs/git-commit-helper
cd git-commit-helper

# 使用安装脚本（推荐）
./install.sh

# 或手动安装
cargo build --release
./target/release/git-commit-helper install
./target/release/git-commit-helper config
```

## 使用方法

### 基础命令

```bash
# 配置 AI 服务
git-commit-helper config

# 显示当前配置
git-commit-helper show

# 安装 Git Hook
git-commit-helper install

# 测试翻译功能
git-commit-helper test -t "这是一个测试消息"

# 翻译指定文本
git-commit-helper translate -t "要翻译的文本"

# 翻译指定文件内容
git-commit-helper translate -f path/to/file

# 生成提交信息建议（新增）
git-commit-helper suggest              # 自动推断提交类型
git-commit-helper suggest -t feat      # 指定提交类型为 feat
```

### AI 服务管理

```bash
# 列出所有已配置的服务
git-commit-helper list

# 服务管理
git-commit-helper service add         # 添加新服务
git-commit-helper service edit        # 修改服务配置
git-commit-helper service remove      # 删除服务
git-commit-helper service set-default # 设置默认服务
```

## 提交类型说明

使用 suggest 命令时，可以通过 -t 选项指定以下类型：

- feat: 新功能
- fix: 修复问题
- docs: 文档变更
- style: 代码格式调整
- refactor: 代码重构
- test: 测试相关
- chore: 构建或辅助工具变更

## 配置说明

配置文件默认保存在：
- Linux: `~/.config/git-commit-helper/config.json`
- macOS: `~/Library/Application Support/git-commit-helper/config.json`
- Windows: `%APPDATA%\git-commit-helper\config.json`

也可以在配置过程中选择自定义路径。

## 工作流程

1. 常规提交流程：
   - 编写提交信息时，如果包含中文内容，工具会自动询问是否需要翻译
   - 确认后会自动翻译并保持中英双语格式

2. AI 辅助提交流程（新增）：
   - 使用 git add 添加需要提交的文件
   - 运行 `git-commit-helper suggest` 生成提交建议
   - 预览生成的提交信息
   - 选择是否需要中英双语格式
   - 确认后自动提交

## 打包说明

本项目支持打包为以下格式：

### Arch Linux 包

```bash
# 使用 PKGBUILD 构建
./install.sh package arch
# 或直接使用
makepkg -sf
```

### Debian 包

```bash
# 使用脚本构建
./install.sh package deb
# 或直接使用
dpkg-buildpackage -us -uc
```

### RPM 包

```bash
# 使用脚本构建
./install.sh package rpm
# 或直接使用
rpmbuild -ba git-commit-helper.spec
```

### 依赖说明

打包工具依赖：
- Arch Linux: `pacman -S base-devel`
- Debian/Ubuntu: `apt install build-essential devscripts debhelper`
- Fedora/RHEL: `dnf install rpm-build rpmdevtools`

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

[MIT](LICENSE)
