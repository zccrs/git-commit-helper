#!/bin/bash
set -e

# 编译项目
cargo build --release

# 安装到当前 git 仓库
./target/release/git-commit-helper install

# 运行配置向导
./target/release/git-commit-helper config

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
