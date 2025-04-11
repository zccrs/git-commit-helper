_git_commit_helper() {
    local i cur prev opts cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}; do
        case "${cmd},${i}" in
            ",$1")
                cmd="git_commit_helper"
                ;;
            git_commit_helper,config)
                cmd="git_commit_helper__config"
                ;;
            git_commit_helper,show)
                cmd="git_commit_helper__show"
                ;;
            git_commit_helper,install)
                cmd="git_commit_helper__install"
                ;;
            git_commit_helper,service)
                cmd="git_commit_helper__service"
                ;;
            git_commit_helper,translate)
                cmd="git_commit_helper__translate"
                ;;
            git_commit_helper,ai-review)
                cmd="git_commit_helper__ai_review"
                ;;
            git_commit_helper__service,add)
                cmd="git_commit_helper__service__add"
                ;;
            git_commit_helper__service,edit)
                cmd="git_commit_helper__service__edit"
                ;;
            git_commit_helper__service,remove)
                cmd="git_commit_helper__service__remove"
                ;;
            git_commit_helper__service,set-default)
                cmd="git_commit_helper__service__set_default"
                ;;
            git_commit_helper__service,set-timeout)
                cmd="git_commit_helper__service__set_timeout"
                ;;
            git_commit_helper__service,list)
                cmd="git_commit_helper__service__list"
                ;;
            git_commit_helper__service,test)
                cmd="git_commit_helper__service__test"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        git_commit_helper)
            opts="config show install service translate commit ai-review help"
            if [[ ${cur} == -* ]] ; then
                opts="--help -h --version -V"
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        git_commit_helper__config)
            COMPREPLY=( $(compgen -W "" -- "${cur}") )
            return 0
            ;;
        git_commit_helper__show)
            COMPREPLY=( $(compgen -W "" -- "${cur}") )
            return 0
            ;;
        git_commit_helper__install)
            opts=""
            if [[ ${cur} == -* ]] ; then
                opts="--path -p --force -f"
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        git_commit_helper__service)
            opts="add edit remove set-default set-timeout list test help"
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        git_commit_helper__service__list)
            COMPREPLY=( $(compgen -W "" -- "${cur}") )
            return 0
            ;;
        git_commit_helper__service__test)
            opts=""
            if [[ ${cur} == -* ]] ; then
                opts="--text -t"
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        git_commit_helper__service__set_timeout)
            opts=""
            if [[ ${cur} == -* ]] ; then
                opts="--seconds -s"
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        git_commit_helper__translate)
            opts=""
            if [[ ${cur} == -* ]] ; then
                opts="--file -f --text -t"
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        git_commit_helper__commit)
            opts=""
            case $prev in
                -t|--type)
                    COMPREPLY=( $(compgen -W "feat fix docs style refactor test chore" -- "$cur") )
                    return
                    ;;
                -m|--message)
                    return
                    ;;
                *)
                    opts="$opts -t --type"
                    opts="$opts -m --message"
                    opts="$opts -a --all"
                    opts="$opts --no-review"
                    ;;
            esac
            ;;
        git_commit_helper__ai_review)
            opts=""
            if [[ ${cur} == -* ]] ; then
                opts="--enable --disable --status"
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

complete -F _git_commit_helper git-commit-helper
