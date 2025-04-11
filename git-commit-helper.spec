Name:           git-commit-helper
Version:        0.2.0
Release:        1%{?dist}
Summary:        帮助规范 git commit message 的工具

License:        MIT
URL:            https://github.com/zccrs/git-commit-helper
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust, cargo
Requires:       git

%description
一个帮助规范 git commit message 的命令行工具

%prep
%setup -q

%build
cargo build --release

%install
install -Dm755 target/release/git-commit-helper %{buildroot}%{_bindir}/git-commit-helper
install -Dm644 completions/git-commit-helper.bash %{buildroot}%{_datadir}/bash-completion/completions/git-commit-helper
install -Dm644 completions/git-commit-helper.zsh %{buildroot}%{_datadir}/zsh/site-functions/_git-commit-helper

%files
%license LICENSE
%{_bindir}/git-commit-helper
%{_datadir}/bash-completion/completions/git-commit-helper
%{_datadir}/zsh/site-functions/_git-commit-helper

%changelog
* Fri Apr 11 2025 zccrs <zccrs@live.com> - 0.2.0-1
- Release version 0.2.0
- Improve translation service stability

* Fri Apr 11 2025 zccrs <zccrs@live.com> - 0.1.0-1
- Initial package
