# Git Commit Helper

一个智能的 Git 提交消息处理工具，可以自动将中文提交消息翻译成英文。

## 功能特性

- 🔄 自动检测并翻译中文提交消息
- 🤖 支持多个 AI 翻译服务：
  - DeepSeek
  - OpenAI (GPT-3.5/4)
  - Claude
  - GitHub Copilot
  - Google Gemini
  - Grok
- 🔄 智能服务回退：当默认服务失败时自动尝试其他可用服务
- ⚙️ 灵活的配置选项：
  - 支持自定义 API 端点
  - 支持设置默认翻译服务
  - 可配置多个备选服务
- 🎯 保留原始中文消息：在英文翻译后附加原始中文内容
- 🔌 简单的安装过程：一键安装到任何 Git 仓库
- 🌐 支持直接翻译文件或文本内容

## 安装

### 1. 使用安装脚本（推荐）

```bash
git clone https://github.com/zccrs/git-commit-helper.git
cd git-commit-helper
./install.sh
```

### 2. 手动安装

```bash
# 1. 克隆仓库
git clone https://github.com/zccrs/git-commit-helper.git
cd git-commit-helper

# 2. 编译项目
cargo build --release

# 3. 安装到当前 Git 仓库
./target/release/git-commit-helper install

# 4. 配置 AI 服务
./target/release/git-commit-helper config
```

## 使用方法

安装完成后，工具会自动处理您的 Git 提交消息。当检测到中文内容时，会提示是否需要翻译：

```bash
git commit -m "添加新功能：支持自动翻译"
# 工具会自动将消息翻译为：
# Add new feature: Support automatic translation
#
# 添加新功能：支持自动翻译
```

### 命令行选项

```bash
git-commit-helper [命令] [选项]

命令：
  config        配置 AI 服务
  show          显示当前配置信息
  install       将工具安装到 Git 仓库
  service       管理 AI 服务配置
    add         添加新的 AI 服务
    edit        修改已有的 AI 服务配置
    remove      删除 AI 服务
    set-default 设置默认 AI 服务
  list          列出所有配置的 AI 服务
  test          测试指定的 AI 服务
  translate     翻译指定的文件或文本内容

选项：
  -h, --help    显示帮助信息
  -V, --version 显示版本信息
```

### 服务管理示例

```bash
# 添加新的 AI 服务
git-commit-helper service add

# 编辑已有服务
git-commit-helper service edit

# 删除服务
git-commit-helper service remove

# 设置默认服务
git-commit-helper service set-default

# 测试服务
git-commit-helper test -t "这是一个测试消息"
```

### 直接翻译

可以使用 translate 命令直接翻译文件或文本内容：

```bash
# 翻译文件内容
git-commit-helper translate -f README.md

# 翻译指定文本
git-commit-helper translate -t "这是一段需要翻译的中文"
```

## 配置文件

配置文件默认存储在以下位置：
- Linux: `~/.config/git-commit-helper/config.json`
- macOS: `~/Library/Application Support/com.githelper.git-commit-helper/config.json`
- Windows: `%APPDATA%\git-commit-helper\config.json`

您也可以在配置过程中选择自定义配置文件位置。

## 实现原理

### 1. 工作流程

1. **Hook 机制**
   - 通过 Git commit-msg hook 拦截提交消息
   - 识别消息中的中文内容
   - 提供交互式确认是否需要翻译

2. **消息处理**
   - 解析提交消息的标题、正文和特殊标记
   - 保持 Git 提交消息格式规范（标题长度、换行等）
   - 在翻译后保留原始中文内容

3. **翻译服务**
   - 支持多个 AI 服务作为翻译后端
   - 实现服务自动回退机制
   - 允许用户手动选择翻译服务

4. **配置管理**
   - 使用 JSON 格式存储配置
   - 支持多服务配置和默认服务设置
   - 提供交互式配置界面

### 2. 代码架构

```
src/
├── main.rs          # 程序入口和命令行处理
├── config.rs        # 配置文件管理
├── git.rs          # Git 相关操作
├── install.rs      # Hook 安装逻辑
├── translator/     # 翻译相关模块
│   ├── mod.rs     # 消息解析和格式化
│   └── ai_service.rs # AI 服务实现
└── debug.rs        # 调试日志功能
```

## 调试指南

### 1. 日志级别控制

设置环境变量来控制日志输出：

```bash
# 显示所有调试信息
RUST_LOG=debug git-commit-helper <command>

# 只显示警告和错误
RUST_LOG=warn git-commit-helper <command>

# 显示详细信息
RUST_LOG=trace git-commit-helper <command>
```

### 2. VS Code 调试

项目包含了完整的 VS Code 调试配置：

1. **配置调试**
   ```bash
   # 1. 打开 VS Code 调试面板
   # 2. 选择 "Debug config" 配置
   # 3. 按 F5 启动调试
   ```

2. **Hook 安装调试**
   ```bash
   # 选择 "Debug install" 配置
   # 使用 VS Code 调试器跟踪安装过程
   ```

3. **提交消息处理调试**
   ```bash
   # 选择 "Debug commit-msg" 配置
   # 输入提交消息文件路径
   # 跟踪消息处理流程
   ```

4. **单元测试调试**
   ```bash
   # 选择 "Debug unit tests" 配置
   # 可以单步调试测试用例
   ```

### 3. 问题排查

1. **翻译服务问题**
   ```bash
   # 测试特定服务
   git-commit-helper test -t "测试文本"
   
   # 查看详细请求日志
   RUST_LOG=debug git-commit-helper test -t "测试文本"
   ```

2. **配置问题**
   ```bash
   # 查看当前配置
   git-commit-helper show
   
   # 验证配置文件位置
   RUST_LOG=debug git-commit-helper show
   ```

3. **Hook 问题**
   ```bash
   # 检查 Hook 安装
   ls -l .git/hooks/commit-msg
   
   # 手动测试 Hook
   echo "测试消息" > .git/COMMIT_EDITMSG
   .git/hooks/commit-msg .git/COMMIT_EDITMSG
   ```

## 高级使用技巧

1. **自定义 API 服务**
   ```bash
   # 添加自定义 API 端点
   git-commit-helper service add
   # 选择服务类型后输入自定义 API 端点
   ```

2. **批量处理**
   ```bash
   # 跳过翻译提示
   git commit --no-verify -m "提交消息"
   
   # 在 CI/CD 中使用
   export GIT_COMMIT_HELPER_AUTO=1
   ```

3. **多仓库配置**
   ```bash
   # 为不同仓库使用不同配置
   GIT_COMMIT_HELPER_CONFIG=/path/to/config.json git-commit-helper install
   ```

## 性能优化建议

1. **服务选择**
   - 优先选择响应速度快的服务作为默认服务
   - 将稳定性高的服务配置为备选服务

2. **配置优化**
   - 使用就近的 API 端点
   - 适当调整超时设置

3. **缓存机制**
   - 程序会缓存配置文件
   - 可以配置多个备选服务以提高可用性

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 常见问题

Q: 如何更改已配置的 API Key？
A: 使用 `git-commit-helper service edit` 命令修改已有服务的配置。

Q: 如何临时禁用翻译？
A: 在提交时使用 `git commit --no-verify` 可以跳过 hook 执行。

Q: 翻译服务失败怎么办？
A: 工具会自动尝试使用其他已配置的服务，您也可以手动选择要使用的备选服务。
