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
- 📋 测试建议
  - 基于代码变更智能生成黑盒测试建议
  - 关注测试重点和覆盖范围
  - 可通过参数禁用
- 📝 产品日志
  - 智能识别用户可感知的功能变化
  - 自动生成面向用户的功能说明
  - 仅在涉及产品功能时添加

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

### ⏳ 网络请求进度提示

为所有慢速网络请求（如 GitHub PR、Gerrit、AI 服务等）增加了进度提示，进度信息会持续在同一行动态刷新，避免用户误以为程序卡死。示例输出：

```
正在请求 github.com 获取PR内容 ...
正在请求 github.com 获取PR内容 100%...
正在请求 api.openai.com 进行代码审查 30%...
```

进度条会根据请求阶段自动更新，所有输出均直接使用 print/println，确保用户一定能看到。


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
| config | 配置 AI 服务 | `git-commit-helper config [--set-only-chinese <true\|false>/--set-only-english <true\|false>]` |
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
| commit | 生成提交信息 | `git-commit-helper commit [-t 类型] [-m 描述] [-a] [--no-review/--no-test-suggestions/--only-chinese/--only-english] [--issues ISSUE...]` |
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
    --no-test-suggestions    禁用当前提交的测试建议功能
    --only-chinese           仅保留中文提交信息
    --only-english           仅保留英文提交信息
    --issues [ISSUE...]      关联多个GitHub issue或PMS链接
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

# 设置默认使用英文
git-commit-helper config --set-only-english true   # 默认仅使用英文

# 设置默认使用中英双语
git-commit-helper config --set-only-chinese false --set-only-english false  # 默认使用中英双语

# 单次提交使用中文
git-commit-helper commit --type feat --message "添加新功能" --only-chinese

# 单次提交使用英文
git-commit-helper commit --type feat --message "Add new functions" --only-english

# 禁用测试建议
git-commit-helper commit --no-test-suggestions

# 同时禁用代码审查和测试建议
git-commit-helper commit --no-review --no-test-suggestions

# 关联GitHub issue
git-commit-helper commit --issues "https://github.com/owner/repo/issues/123"
git-commit-helper commit --issues "123"  # 当前项目的issue
git-commit-helper commit --issues "123" "456" "789"  # 多个issue

# 关联PMS链接  
git-commit-helper commit --issues "https://pms.uniontech.com/bug-view-320461.html"
git-commit-helper commit --issues "https://pms.uniontech.com/task-view-374223.html"
git-commit-helper commit --issues "https://pms.uniontech.com/story-view-38949.html"

# 混合关联
git-commit-helper commit --issues "123" "https://pms.uniontech.com/bug-view-320461.html"
git-commit-helper commit --issues "https://github.com/owner/repo/issues/123" "https://pms.uniontech.com/task-view-374223.html"
```

### AI 代码审查功能

工具提供两种代码审查方式：

1. 本地提交审查：在每次提交代码时自动执行
2. 远程代码审查：支持审查 GitHub 和 Gerrit 上的改动

### 测试建议功能

工具在生成提交信息时会自动包含测试建议部分，帮助开发者明确测试重点：

1. **智能测试建议生成**
   - 基于代码变更自动分析测试需求
   - 专注于黑盒测试方法和策略
   - 提供具体的测试场景和边界条件

2. **测试范围覆盖**
   - 功能性测试建议（正常流程、异常处理）
   - 边界值测试建议（输入验证、数据范围）
   - 安全性测试建议（权限验证、数据保护）
   - 性能测试建议（响应时间、负载处理）

3. **灵活控制**
   - 默认启用测试建议生成
   - 可通过 `--no-test-suggestions` 参数禁用
   - 支持中英双语测试建议

示例生成的测试建议：
```text
feat: 添加用户认证模块

1. 实现基于 JWT 的认证系统
2. 添加用户登录和注册端点
3. 包含使用 bcrypt 的密码哈希处理
4. 设置令牌刷新机制

Log: 新增用户登录注册功能

Influence:
1. 测试用户注册功能，包括有效和无效输入
2. 验证登录功能，测试正确和错误的凭据
3. 测试 JWT 令牌生成和验证流程
4. 验证密码安全性和哈希处理
5. 测试令牌刷新机制和过期处理
6. 验证受保护端点的访问控制
```

### Issue 关联功能

工具支持在提交信息中自动关联 GitHub issue 和 PMS 链接，使提交与相关任务建立明确的关联关系：

1. **GitHub Issue 关联**
   - 支持完整的 GitHub issue URL
   - 支持当前项目的 issue 编号
   - 自动检测是否为同一项目，生成合适的引用格式

2. **PMS 链接关联**
   - 支持联创工程管理系统（PMS）的链接
   - 自动识别 bug、task、story 三种类型
   - 生成标准化的 PMS 引用格式

3. **多链接支持**
   - 支持在一个命令中指定多个链接
   - 可使用空格或逗号分隔多个链接
   - 自动按类型分组合并同类引用

4. **引用字段格式**
   - GitHub: `Fixes: #123` 或 `Fixes: owner/repo#123`
   - 多个 GitHub: `Fixes: #123 #456 owner/repo#789`
   - PMS Bug: `PMS: BUG-320461`
   - PMS Task: `PMS: TASK-374223`
   - PMS Story: `PMS: STORY-38949`
   - 多个 PMS: `PMS: BUG-123 TASK-456 STORY-789`

使用示例：
```bash
# 单个 GitHub issue (当前项目)
git-commit-helper commit --issues "123"
# 生成: Fixes: #123

# 多个 GitHub issue (当前项目)
git-commit-helper commit --issues "123 456 789"
# 生成: Fixes: #123 #456 #789

# 混合不同项目的 GitHub issue
git-commit-helper commit --issues "123 https://github.com/owner/repo/issues/456"
# 生成: Fixes: #123 owner/repo#456

# 多个 PMS 链接
git-commit-helper commit --issues "https://pms.uniontech.com/bug-view-320461.html https://pms.uniontech.com/task-view-374223.html"
# 生成: PMS: BUG-320461 TASK-374223

# 混合 GitHub 和 PMS
git-commit-helper commit --issues "123 https://pms.uniontech.com/bug-view-320461.html"
# 生成:
# Fixes: #123
# PMS: BUG-320461

# 使用逗号分隔
git-commit-helper commit --issues "123,456,789"
# 生成: Fixes: #123 #456 #789

# 混合分隔符
git-commit-helper commit --issues "123 456,https://pms.uniontech.com/task-view-374223.html"
# 生成:
# Fixes: #123 #456
# PMS: TASK-374223
```

### 产品日志功能

工具支持自动生成产品导向的日志字段，帮助产品经理向用户清晰传达功能变化：

1. **智能识别用户功能**
   - 自动判断变更是否涉及用户可感知的功能
   - 仅在真正的产品功能变化时生成Log字段
   - 过滤纯技术性或内部实现的修改

2. **面向用户的表达**
   - 使用用户易懂的语言描述功能变化
   - 专注于功能价值而非技术细节
   - 适合产品发布说明和用户沟通

3. **应用场景**
   - 新功能发布：`Log: 新增深色模式主题`
   - UI改进：`Log: 优化搜索界面交互体验`
   - 功能修复：`Log: 修复无法保存文件的问题`
   - 设置增强：`Log: 支持设置鼠标光标大小`

4. **不生成Log的情况**
   - 代码重构或架构调整
   - 依赖库更新或版本升级
   - 内部工具或开发环境配置
   - 纯技术性能优化（用户无感知）

使用示例：
```bash
# 添加用户功能时会生成Log字段
git-commit-helper commit
# 可能生成:
# feat: add dark mode theme support
#
# 1. Implement theme switching mechanism
# 2. Add dark mode color scheme
# 3. Update all UI components for theme support
#
# Log: 新增深色模式主题
#
# Influence:
# 1. Test theme switching in different scenarios
# 2. Verify color contrast meets accessibility standards

# 技术重构时不会生成Log字段
git-commit-helper commit
# 可能生成:
# refactor: optimize database query performance
#
# 1. Replace N+1 queries with batch loading
# 2. Add database connection pooling
# 3. Optimize slow query indexes
#
# Influence:
# 1. Test query performance under load
# 2. Verify data consistency after optimization
```

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
```txt
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

## 🚀 自动推送到 Arch Linux AUR

本项目支持通过 GitHub Actions 自动将 PKGBUILD 及相关文件推送到 AUR 仓库，实现一键同步更新。

### 配置方法

1. **在 GitHub 仓库设置 Secrets：**
   - 添加名为 `AUR_SSH_PRIVATE_KEY` 的 secret，内容为你的 AUR 账户 SSH 私钥（建议使用专用密钥，且设置只读权限）。
   - 可选：如需自定义 AUR 仓库地址，添加 `AUR_REPO_URL` secret。

     **获取方法：**
     1. 登录 [AUR 官网](https://aur.archlinux.org/) 并搜索你的包名。
     2. 打开你的包页面，点击右上角 “Git Clone” 按钮，会显示类似：
        ```
        git clone ssh://aur@aur.archlinux.org/your-aur-repo.git
        ```
     3. 复制 `ssh://aur@aur.archlinux.org/your-aur-repo.git` 作为 `AUR_REPO_URL` 的值。

2. **GitHub Actions Workflow 示例：**

```yaml
jobs:
  aur-publish:
    runs-on: ubuntu-latest
    if: github.ref_type == 'tag'
    steps:
      - uses: actions/checkout@v4
      - name: Set up SSH
        run: |
          mkdir -p ~/.ssh
          echo "${{ secrets.AUR_SSH_PRIVATE_KEY }}" > ~/.ssh/id_rsa
          chmod 600 ~/.ssh/id_rsa
          ssh-keyscan aur.archlinux.org >> ~/.ssh/known_hosts
      - name: Clone AUR repo
        run: |
          git clone "${{ secrets.AUR_REPO_URL || 'ssh://aur@aur.archlinux.org/<your-aur-repo>.git' }}" aur-repo
      - name: Update PKGBUILD and files
        run: |
          cp PKGBUILD aur-repo/
          # 如有其它 AUR 文件一并复制
          cd aur-repo
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add PKGBUILD
          git commit -m "chore: update PKGBUILD to ${{ github.ref_name }}"
          git push origin master
```

3. **注意事项：**
   - 推送前请确保 PKGBUILD、.SRCINFO 等文件已更新到最新版本。
   - 推荐在发布 tag 时自动推送，避免开发分支误同步。
   - 请妥善保管 SSH 私钥，避免泄露。

4. **常见问题：**
   - 如遇权限或认证失败，请检查 SSH 密钥权限及 AUR 账户设置。
   - 若需同步其它文件，请在 workflow 中补充 `cp` 和 `git add` 命令。

## ⚙️ 配置文件

默认配置路径：
- 🐧 Linux: `~/.config/git-commit-helper/config.json`
- 🍎 macOS: `~/Library/Application Support/git-commit-helper/config.json`
- 🪟 Windows: `%APPDATA%\git-commit-helper\config.json`

## 📝 版本历史

### v0.5.3

- Release version 0.5.3

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
