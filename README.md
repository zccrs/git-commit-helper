<h1 align="center">Git Commit Helper</h1>

<p align="center">
  一个强大的 Git 提交消息助手，支持多个 AI 服务，实现智能提交消息生成和中英互译
</p>

<p align="center">
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="license"/>
  </a>
  <a href="https://github.com/rust-lang/rust">
    <img src="https://img.shields.io/badge/rust-stable-orange.svg" alt="rust"/>
  </a>
</p>

## ✨ 功能特点

- 🤖 多 AI 服务支持
  - DeepSeek (已测试)
  - OpenAI/GPT (已测试)
  - Claude (未测试)
  - Github Copilot (已测试)
  - Google Gemini (已测试)
  - Grok (已测试)
  - Qwen (已测试)

  > 注意：目前仅 Claude 服务尚未经过完整测试。如果您在使用此服务时遇到问题，欢迎反馈。

- 🔧 高度可定制
  - 支持自定义 API 地址
  - 支持自定义 AI 模型
  - 支持服务优先级配置
- 📝 智能提交
  - 自动生成规范的提交信息
  - 支持指定提交类型
  - AI 分析代码变更内容
- 🔍 智能代码审查
  - 自动审查代码变更
  - 性能和安全建议
  - 可通过参数禁用
- 🌏 中英双语
  - 自动检测中文内容
  - 智能中英互译
  - 保持格式规范

## 📦 安装

### 从源码安装

```bash
# 克隆仓库
git clone https://github.com/zccrs/git-commit-helper
cd git-commit-helper

# 快速安装（推荐）
./install.sh
```

### 包管理器安装

```bash
# Arch Linux
yay -S git-commit-helper

# Debian/Ubuntu
sudo apt install git-commit-helper

# Fedora
sudo dnf install git-commit-helper
```

### Debian/Ubuntu

添加软件源：
```bash
# 添加GPG公钥（暂未实现）
# curl -fsSL https://zccrs.github.io/git-commit-helper/KEY.gpg | sudo gpg --dearmor -o /usr/share/keyrings/git-commit-helper.gpg

# 添加软件源
echo "deb [trusted=yes] https://zccrs.github.io/git-commit-helper/ stable main" | sudo tee /etc/apt/sources.list.d/git-commit-helper.list

# 更新软件源并安装
sudo apt update
sudo apt install git-commit-helper
```

## 🚀 快速开始

1. 配置 AI 服务
```bash
git-commit-helper config
```

2. 安装 Git Hook
```bash
git-commit-helper install
```

3. 开始使用
```bash
# 智能生成提交信息
git add .
git-commit-helper commit

# 或者手动编写并自动翻译
git commit
```

## 💡 使用指南

### 翻译示例

以下是一个实际的提交消息翻译示例：

<details>
<summary>展开查看示例</summary>

原始提交消息：
```text
支持将原中文内容自动换行处理

如原中文内容是："我是中文commit信息"，在翻译处理后，
可能会变成：
"我是中文\ncommit信息"，这取决于一行的长度
否超出git的推荐值。
```

翻译后的提交消息：
```text
Support automatic line breaking for the original Chinese content

If the original Chinese content is: "我是中文commit信息", after
translation,
it may become:
"我是中文\ncommit信息", depending on whether the length of
a line exceeds the recommended value of git.

支持将原中文内容自动换行处理

如原中文内容是："我是中文commit信息"，在翻译处理后，
可能会变成：
"我是中文\ncommit信息"，这取决于一行的长度
否超出git的推荐值。
```
</details>

### 翻译命令

使用翻译命令有三种方式：
```bash
# 方式1：指定文件路径
git-commit-helper translate -f path/to/file

# 方式2：指定文本内容
git-commit-helper translate -t "要翻译的文本"

# 方式3：智能判断（推荐）
git-commit-helper translate "要翻译的文本"              # 文本内容
git-commit-helper translate /path/to/existing/file    # 文件路径
```

命令会自动判断参数内容：如果是一个存在的文件路径则读取文件内容进行翻译，否则将参数内容作为文本进行翻译。

### 命令概览

| 命令 | 说明 | 示例 |
|------|------|------|
| config | 配置 AI 服务 | `git-commit-helper config [--set-only-chinese <true|false>]` |
| show | 显示当前配置 | `git-commit-helper show` |
| install | 安装 Git Hook | `git-commit-helper install [-f]` |
| ai add | 添加 AI 服务 | `git-commit-helper ai add` |
| ai edit | 编辑 AI 服务配置 | `git-commit-helper ai edit` |
| ai remove | 删除 AI 服务 | `git-commit-helper ai remove` |
| ai set-default | 设置默认服务 | `git-commit-helper ai set-default` |
| ai set-timeout | 设置请求超时 | `git-commit-helper ai set-timeout -s 30` |
| ai list | 列出所有服务 | `git-commit-helper ai list` |
| ai test | 测试指定服务 | `git-commit-helper ai test [-t "测试文本"]` |
| translate | 翻译内容 | `git-commit-helper translate [-f 文件] [-t 文本]` |
| commit | 生成提交信息 | `git-commit-helper commit [-t 类型] [-m 描述] [-a] [--no-review]` |
| ai-review | 管理 AI 代码审查 | `git-commit-helper ai-review [--enable/--disable/--status]` |

### 提交类型

| 类型 | 说明 | 使用场景 |
|------|------|----------|
| feat | 新功能 | 添加新特性 |
| fix | 修复问题 | 修复 bug |
| docs | 文档更新 | 更新文档 |
| style | 格式调整 | 不影响代码逻辑的格式修改 |
| refactor | 代码重构 | 不修复问题也不添加特性的代码更改 |
| test | 测试相关 | 添加或修改测试用例 |
| chore | 其他更新 | 构建过程或辅助工具的变更 |

### 命令行参数

```bash
# 配置
git-commit-helper config [选项]
    --set-only-chinese <true|false>  设置默认是否只使用中文提交信息

# 远程代码审查
git-commit-helper <URL>
    支持以下代码平台的改动审查：

    1. GitHub
    - PR: https://github.com/owner/repo/pull/123
    - Commit: https://github.com/owner/repo/commit/hash

    2. Gerrit
    - Change: https://gerrit.uniontech.com/c/udcp/udcp-uim/+/179042

# AI 代码审查管理
git-commit-helper ai-review [选项]
    --enable           全局启用代码审查功能
    --disable         全局禁用代码审查功能
    --status          查看代码审查功能的当前状态

# 生成提交信息
git-commit-helper commit [选项]
    -t, --type <TYPE>         指定提交类型 (可选)
    -m, --message <MSG>       提供对改动的描述 (可选)
    -a, --all                 自动添加所有已修改但未暂存的文件
    --no-review              禁用当前提交的代码审查功能
    --only-chinese           仅保留中文提交信息
```

示例：
```bash
# 生成提交信息
git-commit-helper commit

# 指定提交类型
git-commit-helper commit --type feat

# 提供改动描述
git-commit-helper commit --message "修复了用户无法登录的问题"

# 自动添加所有修改并提交
git-commit-helper commit -a

# 完整示例
git-commit-helper commit --type fix --message "修复内存泄漏" -a

# 设置默认使用中文
git-commit-helper config --set-only-chinese true   # 默认仅使用中文
git-commit-helper config --set-only-chinese false  # 默认使用中英双语

# 单次提交使用中文
git-commit-helper commit --type feat --message "添加新功能" --only-chinese
```

### AI 代码审查功能

工具提供两种代码审查方式：

1. 本地提交审查：在每次提交代码时自动执行
2. 远程代码审查：支持审查 GitHub 和 Gerrit 上的改动

远程代码审查功能包含：
1. 提交信息翻译
   - 显示原始提交标题和内容
   - 自动检测英文内容并翻译成中文
   - 支持 PR 描述、commit message 等
   - 保持原始格式的同时提供翻译

2. 代码变更审查
   - 代码质量和可维护性评估
   - 潜在问题或漏洞提示
   - 性能影响分析
   - 对现有功能的影响评估
   - 最佳实践建议
   - 具体的改进建议

远程审查支持的平台：
- GitHub
  - Pull Request 审查（支持 PR 标题和描述的翻译）
  - Commit 审查（支持 commit message 的翻译）
- Gerrit
  - Change 审查（支持完整 commit message 的翻译）
  - 支持变更描述、Log、Influence 等信息的翻译

示例：
```bash
# 审查 GitHub PR
git-commit-helper https://github.com/owner/repo/pull/123

# 审查 GitHub commit
git-commit-helper https://github.com/owner/repo/commit/hash

# 审查 Gerrit change
git-commit-helper https://gerrit.example.com/c/project/+/123456
```

输出格式：
```
标题：<原始标题>
中文翻译：<标题翻译>

描述：
<原始描述>

中文翻译：
<描述翻译>

代码审查报告：
...（详细的代码审查内容）
```

#### 本地提交审查

你可以通过以下方式控制代码审查功能：

1. 全局控制（影响所有后续提交）：
```bash
# 启用代码审查
git-commit-helper ai-review --enable

# 禁用代码审查
git-commit-helper ai-review --disable

# 查看当前状态
git-commit-helper ai-review --status
```

2. 单次提交控制（仅影响当前提交）：
```bash
# 提交时临时禁用代码审查
git-commit-helper commit --no-review

# 或者在编辑提交信息时禁用
git commit --no-review
```

## 🔄 版本更新流程

更新版本时需要修改以下文件：

1. **Cargo.toml**
   ```toml
   [package]
   version = "x.y.z"  # 更新版本号
   ```

2. **debian/changelog 和 git-commit-helper.spec**

   注意：debian 和 rpm 包的 changelog 都需要更新，并且内容要保持一致。

   对于 debian/changelog：
   ```
   git-commit-helper (x.y.z) unstable; urgency=medium

   * 此处列出从上一版本到当前版本的所有提交记录，可以通过以下命令获取：
   git log <上一版本>..HEAD --oneline

   按类型整理提交记录，例如：
   * feat: 添加的新功能
   * fix: 修复的问题
   * docs: 文档更新
   * chore: 其他修改

   -- 作者 <邮箱>  `date "+%a, %d %b %Y %H:%M:%S %z"`  # 使用系统当前时间
   ```

   对于 git-commit-helper.spec：
   ```
   %changelog
   * 发布日期 作者 <邮箱> - x.y.z-1
   # 此处列出与 debian/changelog 相同的更新内容，保持格式一致：
   - feat: 添加的新功能
   - fix: 修复的问题
   - docs: 文档更新
   - chore: 其他修改
   ```

3. **PKGBUILD**
   ```bash
   pkgver=x.y.z  # 更新版本号
   ```

4. **git-commit-helper.spec**
   ```spec
   Version:        x.y.z  # 更新版本号

   # 在 %changelog 部分添加新版本信息
   * 发布日期 作者 <邮箱> - x.y.z-1
   - Release version x.y.z
   - 更新内容描述...
   ```

5. **Git 标签**
   ```bash
   # 提交更改
   git add .
   git commit -m "chore: bump version to x.y.z"

   # 创建新标签
   git tag -a vx.y.z -m "Release version x.y.z"

   # 推送更改和标签
   git push origin master
   git push origin vx.y.z
   ```

## 📂 项目结构

```
src/
├── ai_service.rs    # AI 服务实现
├── auth/           # 认证相关模块
├── commit.rs       # 提交消息处理
├── config.rs       # 配置管理
├── debug.rs        # 调试工具
├── gerrit.rs       # Gerrit 集成
├── github.rs       # GitHub 集成
├── git.rs          # Git 操作
├── install.rs      # 安装工具
├── lib.rs          # 库入口
├── main.rs         # 主程序
└── review.rs       # 代码审查
```

## 📦 项目打包

```bash
# 打包脚本使用方法
./install.sh package [arch|deb|rpm]

# 手动打包
makepkg -sf          # Arch Linux
dpkg-buildpackage    # Debian
rpmbuild -ba *.spec  # RPM
```

## ⚙️ 配置文件

默认配置路径：
- 🐧 Linux: `~/.config/git-commit-helper/config.json`
- 🍎 macOS: `~/Library/Application Support/git-commit-helper/config.json`
- 🪟 Windows: `%APPDATA%\git-commit-helper\config.json`

## 🔍 调试日志

通过设置环境变量 `RUST_LOG` 可以开启调试日志：

```bash
# 开启全部调试日志
export RUST_LOG=debug
git-commit-helper ...

# 或者在运行时临时开启
RUST_LOG=debug git-commit-helper ...

# 仅开启特定模块的调试日志
RUST_LOG=git_commit_helper=debug git-commit-helper ...
```

常用日志级别：
- error: 仅显示错误
- warn: 显示警告和错误
- info: 显示基本信息（默认）
- debug: 显示调试信息
- trace: 显示所有信息

## 🤝 贡献

欢迎提交 [Issue](../../issues) 和 [Pull Request](../../pulls)！

## 📄 许可证

本项目采用 [MIT](LICENSE) 许可证。
