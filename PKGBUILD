pkgname=git-commit-helper
pkgver=0.1.0.r22
pkgrel=1
pkgdesc="一个帮助规范 git commit message 的工具"
arch=('x86_64')
url="https://github.com/zccrs/git-commit-helper"
license=('MIT')
depends=('git')
makedepends=('rust' 'cargo')

# 直接使用当前目录作为源
source=()
sha256sums=()

pkgver() {
    cd "$startdir"
    printf "0.1.0.r%s" "$(git rev-list --count HEAD)"
}

prepare() {
    cd "$startdir"
    export CARGO_HOME="$srcdir/cargo-home"
    cargo fetch --locked || true
}

build() {
    cd "$startdir"
    export CARGO_HOME="$srcdir/cargo-home"
    RUSTUP_TOOLCHAIN=stable cargo build --release
}

check() {
    cd "$startdir"
    export CARGO_HOME="$srcdir/cargo-home"
    RUSTUP_TOOLCHAIN=stable cargo test --release || true
}

package() {
    cd "$startdir"
    
    # 创建必要的目录
    mkdir -p "$pkgdir/usr/bin"
    mkdir -p "$pkgdir/usr/share/bash-completion/completions"
    mkdir -p "$pkgdir/usr/share/zsh/site-functions"
    
    # 安装二进制文件
    install -Dm755 "target/release/git-commit-helper" "$pkgdir/usr/bin/git-commit-helper"
    
    # 安装补全文件（使用固定路径）
    if [ -f "completions/git-commit-helper.bash" ]; then
        install -Dm644 "completions/git-commit-helper.bash" \
            "$pkgdir/usr/share/bash-completion/completions/git-commit-helper"
    else
        echo "Warning: Bash completion file not found"
    fi
    
    if [ -f "completions/git-commit-helper.zsh" ]; then
        install -Dm644 "completions/git-commit-helper.zsh" \
            "$pkgdir/usr/share/zsh/site-functions/_git-commit-helper"
    else
        echo "Warning: Zsh completion file not found"
    fi
    
    # 安装许可证文件
    if [ -f LICENSE ]; then
        install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    fi
}
