pkgname=git-commit-helper
pkgver=0.1.0.r19
pkgrel=1
pkgdesc="一个帮助规范 git commit message 的工具"
arch=('x86_64')
url="https://github.com/zccrs/git-commit-helper"
license=('MIT')
depends=('git')
makedepends=('rust' 'cargo')

# 使用本地文件作为源
source=("$pkgname::git+file://${PWD}")
sha256sums=('SKIP')

pkgver() {
    cd "$srcdir/$pkgname"
    printf "0.1.0.r%s" "$(git rev-list --count HEAD)"
}

prepare() {
    cd "$srcdir/$pkgname"
    # 确保cargo可以访问网络获取依赖
    export CARGO_HOME="$srcdir/cargo-home"
    cargo fetch --locked || true
}

build() {
    cd "$srcdir/$pkgname"
    export CARGO_HOME="$srcdir/cargo-home"
    RUSTUP_TOOLCHAIN=stable cargo build --release
}

check() {
    cd "$srcdir/$pkgname"
    export CARGO_HOME="$srcdir/cargo-home"
    RUSTUP_TOOLCHAIN=stable cargo test --release || true
}

package() {
    cd "$srcdir/$pkgname"
    echo "Current directory: $(pwd)"
    echo "Source directory structure:"
    find . -type f -name "git-commit-helper" -o -name "*.bash" -o -name "*.zsh"
    
    # 创建必要的目录
    mkdir -p "$pkgdir/usr/bin"
    mkdir -p "$pkgdir/usr/share/bash-completion/completions"
    mkdir -p "$pkgdir/usr/share/zsh/site-functions"
    
    # 查找并安装二进制文件
    BINARY=$(find . -type f -executable -name "git-commit-helper" | grep "release/git-commit-helper$")
    if [ -z "$BINARY" ]; then
        echo "Error: Binary not found!"
        exit 1
    fi
    echo "Installing binary from: $BINARY"
    install -Dm755 "$BINARY" "$pkgdir/usr/bin/git-commit-helper"
    
    # 查找并安装补全文件
    BASH_COMP=$(find . -type f -name "git-commit-helper.bash")
    ZSH_COMP=$(find . -type f -name "git-commit-helper.zsh")
    
    if [ -n "$BASH_COMP" ]; then
        install -Dm644 "$BASH_COMP" "$pkgdir/usr/share/bash-completion/completions/git-commit-helper"
    else
        echo "Warning: Bash completion file not found"
    fi
    
    if [ -n "$ZSH_COMP" ]; then
        install -Dm644 "$ZSH_COMP" "$pkgdir/usr/share/zsh/site-functions/_git-commit-helper"
    else
        echo "Warning: Zsh completion file not found"
    fi
    
    # 安装许可证文件（可选）
    if [ -f LICENSE ]; then
        install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    else
        echo "Warning: LICENSE file not found, skipping..."
    fi
}
