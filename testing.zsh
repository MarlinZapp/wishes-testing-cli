#compdef testing

autoload -U is-at-least

_testing() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-s+[Here you can provide a shell if you don'\''t want to use the environment variable SHELL.]: :(bash elvish fish powershell zsh)' \
'--shell=[Here you can provide a shell if you don'\''t want to use the environment variable SHELL.]: :(bash elvish fish powershell zsh)' \
'-h[Print help]' \
'--help[Print help]' \
":: :_testing_commands" \
"*::: :->testing" \
&& ret=0
    case $state in
    (testing)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:testing-command-$line[1]:"
        case $line[1] in
            (generate)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(case)
_arguments "${_arguments_options[@]}" : \
'-e+[The path to the executable starting the surrealdb server]: :_files' \
'--surrealdb-executable=[The path to the executable starting the surrealdb server]: :_files' \
'-h[Print help]' \
'--help[Print help]' \
":: :_testing__case_commands" \
"*::: :->case" \
&& ret=0

    case $state in
    (case)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:testing-case-command-$line[1]:"
        case $line[1] in
            (one)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of users to register. Defaults to 1000.]: :_default' \
'--n-users=[Number of users to register. Defaults to 1000.]: :_default' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(two)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of wishes to create. Defaults to 1000.]: :_default' \
'--n-wishes=[Number of wishes to create. Defaults to 1000.]: :_default' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_testing__case__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:testing-case-help-command-$line[1]:"
        case $line[1] in
            (one)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(two)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_testing__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:testing-help-command-$line[1]:"
        case $line[1] in
            (generate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(case)
_arguments "${_arguments_options[@]}" : \
":: :_testing__help__case_commands" \
"*::: :->case" \
&& ret=0

    case $state in
    (case)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:testing-help-case-command-$line[1]:"
        case $line[1] in
            (one)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(two)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_testing_commands] )) ||
_testing_commands() {
    local commands; commands=(
'generate:Generate shell completions' \
'case:Run a test case' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'testing commands' commands "$@"
}
(( $+functions[_testing__case_commands] )) ||
_testing__case_commands() {
    local commands; commands=(
'one:Run test case one\: Register n users.' \
'two:Run test case one\: Register 10 users and create n/10 wishes.' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'testing case commands' commands "$@"
}
(( $+functions[_testing__case__help_commands] )) ||
_testing__case__help_commands() {
    local commands; commands=(
'one:Run test case one\: Register n users.' \
'two:Run test case one\: Register 10 users and create n/10 wishes.' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'testing case help commands' commands "$@"
}
(( $+functions[_testing__case__help__help_commands] )) ||
_testing__case__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'testing case help help commands' commands "$@"
}
(( $+functions[_testing__case__help__one_commands] )) ||
_testing__case__help__one_commands() {
    local commands; commands=()
    _describe -t commands 'testing case help one commands' commands "$@"
}
(( $+functions[_testing__case__help__two_commands] )) ||
_testing__case__help__two_commands() {
    local commands; commands=()
    _describe -t commands 'testing case help two commands' commands "$@"
}
(( $+functions[_testing__case__one_commands] )) ||
_testing__case__one_commands() {
    local commands; commands=()
    _describe -t commands 'testing case one commands' commands "$@"
}
(( $+functions[_testing__case__two_commands] )) ||
_testing__case__two_commands() {
    local commands; commands=()
    _describe -t commands 'testing case two commands' commands "$@"
}
(( $+functions[_testing__generate_commands] )) ||
_testing__generate_commands() {
    local commands; commands=()
    _describe -t commands 'testing generate commands' commands "$@"
}
(( $+functions[_testing__help_commands] )) ||
_testing__help_commands() {
    local commands; commands=(
'generate:Generate shell completions' \
'case:Run a test case' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'testing help commands' commands "$@"
}
(( $+functions[_testing__help__case_commands] )) ||
_testing__help__case_commands() {
    local commands; commands=(
'one:Run test case one\: Register n users.' \
'two:Run test case one\: Register 10 users and create n/10 wishes.' \
    )
    _describe -t commands 'testing help case commands' commands "$@"
}
(( $+functions[_testing__help__case__one_commands] )) ||
_testing__help__case__one_commands() {
    local commands; commands=()
    _describe -t commands 'testing help case one commands' commands "$@"
}
(( $+functions[_testing__help__case__two_commands] )) ||
_testing__help__case__two_commands() {
    local commands; commands=()
    _describe -t commands 'testing help case two commands' commands "$@"
}
(( $+functions[_testing__help__generate_commands] )) ||
_testing__help__generate_commands() {
    local commands; commands=()
    _describe -t commands 'testing help generate commands' commands "$@"
}
(( $+functions[_testing__help__help_commands] )) ||
_testing__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'testing help help commands' commands "$@"
}

if [ "$funcstack[1]" = "_testing" ]; then
    _testing "$@"
else
    compdef _testing testing
fi
