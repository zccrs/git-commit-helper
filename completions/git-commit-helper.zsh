#compdef git-commit-helper

autoload -U is-at-least

_git-commit-helper() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" \
        '-h[显示帮助]' \
        '--help[显示帮助]' \
        '-V[显示版本]' \
        '--version[显示版本]' \
        "1: :->cmds" \
        "*::arg:->args"

    case "$state" in
        cmds)
            _values 'git-commit-helper 命令' \
                'config[配置 AI 服务]' \
                'show[显示当前配置信息]' \
                'install[将工具安装到当前 git 仓库]' \
                'ai[管理 AI 服务配置]' \
                'translate[翻译中文内容为英文]' \
                'commit[生成提交信息]' \
                'ai-review[管理 AI 代码审查功能]'
            ;;
        args)
            case $line[1] in
                install)
                    _arguments \
                        '-p[指定 git 仓库路径]' \
                        '--path[指定 git 仓库路径]' \
                        '-f[强制安装]' \
                        '--force[强制安装]'
                    ;;
                ai)
                    _values 'ai 命令' \
                        'add[添加新的 AI 服务]' \
                        'edit[修改已有的 AI 服务配置]' \
                        'remove[删除 AI 服务]' \
                        'set-default[设置默认 AI 服务]' \
                        'set-timeout[设置网络请求超时时间]' \
                        'list[列出所有AI服务]' \
                        'test[测试指定的AI服务]'

                    case $line[2] in
                        test)
                            _arguments \
                                '-t[测试用的中文文本]' \
                                '--text[测试用的中文文本]'
                            ;;
                        set-timeout)
                            _arguments \
                                '-s[超时时间（单位：秒）]' \
                                '--seconds[超时时间（单位：秒）]'
                            ;;
                    esac
                    ;;
                translate)
                    _arguments \
                        '-f[要翻译的文件路径]' \
                        '--file[要翻译的文件路径]' \
                        '-t[要翻译的文本内容]' \
                        '--text[要翻译的文本内容]'
                    ;;
                commit)
                    _arguments \
                        '-t[提交消息的类型]' \
                        '--type[提交消息的类型]' \
                        '-m[用户对改动的描述]' \
                        '--message[用户对改动的描述]' \
                        '-a[自动添加所有已修改但未暂存的文件]' \
                        '--all[自动添加所有已修改但未暂存的文件]' \
                        '--no-review[禁用当前提交的代码审查功能]' \
                        '--no-translate[不翻译提交信息]' \
                        '--only-chinese[仅保留中文提交信息]'
                    ;;
                ai-review)
                    _arguments \
                        '--enable[启用 AI 代码审查]' \
                        '--disable[禁用 AI 代码审查]' \
                        '--status[显示当前 AI 代码审查状态]'
                    ;;
            esac
            ;;
    esac
}

_git-commit-helper
