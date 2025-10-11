Name:           git-commit-helper
Version:        0.8.0
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
* 五 10月 11 2025 zccrs <zhangjide@deepin.org> - 0.8.0-1
- Version bump to 0.8.0

* 二 7月 29 2025 zccrs <zccrs@live.com> - 0.7.0-1
- feat: add --no-log parameter and rename --no-test-suggestions to --no-influence
- fix: support multiple arguments for --issues parameter
- feat: add Log field for product-oriented change summaries
- feat: change testing suggestions label to Influence
- feat: support multiple issues in --issues parameter
- feat: add test suggestion disable flag
- Multiple workflow and build improvements

* Mon Jan 06 2025 zccrs <zccrs@live.com> - 0.6.0-1
- Fix variable expansion in PKGBUILD modification for AUR publishing
- Improve GitHub Actions workflow for better package building

* Mon Jun 30 2025 zccrs <zhangjide@deepin.org> - 0.5.3-1
- Release version 0.5.3

* Tue Apr 30 2025 zccrs <zhangjide@deepin.org> - 0.5.0-1
- Release version 0.5.0
* Tue Apr 30 2025 zccrs <zhangjide@deepin.org> - 0.4.3-1
- Release version 0.4.3
* Mon Apr 28 2025 zccrs <zhangjide@deepin.org> - 0.4.2-1
- Release version 0.4.2
* Mon Apr 28 2025 zccrs <zhangjide@deepin.org> - 0.4.1-1
- Release version 0.4.1
* Mon Apr 28 2025 zccrs <zhangjide@deepin.org> - 0.4.0-1
- Release version 0.4.0
* Fri Apr 25 2025 zccrs <zhangjide@deepin.org> - 0.3.3-1
- feat: Show default API endpoint when configuring AI service
- fix: resolve Gemini service integration issues
- fix: use new service for AI service testing
- fix: update GitHub token error message
- fix: Chinese content in the translated commit message now supports automatic wrapping
- chore: bump version to 0.3.3

* Fri Apr 11 2025 zccrs <zhangjide@deepin.org> - 0.3.2-1
- Release version 0.3.2

* Fri Apr 11 2025 zccrs <zhangjide@deepin.org> - 0.3.1-1
- Correct formatting in comments for clarity
- Optimize Arch Linux package version fetching
- Correct Arch Linux package version error

* Fri Apr 11 2025 zccrs <zccrs@live.com> - 0.3.0-1
- Release version 0.3.0
- Add AI code review functionality
- Add option to skip code review
- Optimize commit message generation process
- Improve translation service stability
- Fix multiple code review trigger issue
- Enhance multi-platform build workflow
- Update AI service testing status documentation

* Fri Apr 11 2025 zccrs <zccrs@live.com> - 0.2.1-1
- Release version 0.2.1

* Fri Apr 11 2025 zccrs <zccrs@live.com> - 0.2.0-1
- Release version 0.2.0
- Improve translation service stability

* Fri Apr 11 2025 zccrs <zccrs@live.com> - 0.1.0-1
- Initial package
