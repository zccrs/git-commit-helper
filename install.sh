#!/bin/bash
set -e

# 检查系统是否支持apt命令
check_apt_system() {
    command -v apt >/dev/null 2>&1
}

# 检查rust版本
check_rust_version() {
    local required_version="1.70.0"
    if ! command -v rustc >/dev/null 2>&1; then
        echo "未安装 Rust"
        return 1
    fi

    local current_version=$(rustc --version | cut -d' ' -f2)
    if printf '%s\n%s' "$current_version" "$required_version" | sort -V | head -n1 | grep -q "$required_version"; then
        return 0
    else
        echo "当前 Rust 版本 ($current_version) 低于所需版本 ($required_version)"
        return 1
    fi
}

# 使用apt安装rustup
install_rustup_via_apt() {
    echo "通过系统包管理器安装 rustup..."
    sudo apt-get update && sudo apt-get install -y rustup || return 1
    return 0
}

# 从官网下载安装rustup
install_rustup_from_web() {
    echo "从官网下载 rustup..."
    if command -v wget >/dev/null 2>&1; then
        wget -O /tmp/rustup.sh --progress=bar:force:noscroll --show-progress https://sh.rustup.rs
        echo "安装 rustup..."
        sh /tmp/rustup.sh -y
        rm -f /tmp/rustup.sh
    else
        echo "使用 curl 下载..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    fi
}

# 使用rustup安装rust
install_rust() {
    # 如果rustup不存在，先安装rustup
    if ! command -v rustup >/dev/null 2>&1; then
        if check_apt_system; then
            # 优先使用apt安装rustup
            if ! install_rustup_via_apt; then
                echo "通过apt安装rustup失败，尝试从官网下载安装..."
                install_rustup_from_web
            fi
        else
            install_rustup_from_web
        fi
        source "$HOME/.cargo/env"
    fi

    echo "安装所需的 Rust 版本..."
    rustup install 1.70.0
    rustup default 1.70.0
}

# 在apt系统上检查并安装rust
if check_apt_system; then
    if ! check_rust_version; then
        echo "在apt系统上检测到 Rust 版本不符合要求，将使用 rustup 安装指定版本..."
        install_rust
    fi
fi

# 编译项目
cargo build --release

# 创建本地二进制目录
mkdir -p ~/.local/bin

# 安装二进制文件
cp ./target/release/git-commit-helper ~/.local/bin/
chmod +x ~/.local/bin/git-commit-helper

# 安装到当前 git 仓库
~/.local/bin/git-commit-helper install

# 运行配置向导
~/.local/bin/git-commit-helper config

echo "二进制文件已安装到: ~/.local/bin/git-commit-helper"

# 创建补全文件目录
mkdir -p ~/.local/share/bash-completion/completions
mkdir -p ~/.local/share/zsh/site-functions

# 安装补全文件
cp completions/git-commit-helper.bash ~/.local/share/bash-completion/completions/git-commit-helper
cp completions/git-commit-helper.zsh ~/.local/share/zsh/site-functions/_git-commit-helper

echo "补全文件已安装到:"
echo "  bash: ~/.local/share/bash-completion/completions/git-commit-helper"
echo "  zsh:  ~/.local/share/zsh/site-functions/_git-commit-helper"
echo "请重新加载 shell 配置文件以启用补全功能"
echo "  bash: source ~/.bashrc"
echo "  zsh:  source ~/.zshrc"

echo "安装完成！"

# 添加打包功能
if [ "$1" = "package" ]; then
    case "$2" in
        "arch")
            makepkg -sf
            ;;
        "deb")
            dpkg-buildpackage -us -uc
            ;;
        "rpm")
            rpmbuild -ba git-commit-helper.spec
            ;;
        *)
            echo "用法: $0 package [arch|deb|rpm]"
            exit 1
            ;;
    esac
fi
