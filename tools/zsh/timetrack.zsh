# Helper functions to work with timetrack under zsh

alias tt='timetrack -f "${TIMETRACK_FILE}"'
alias ttl="tt summary last"
alias ttt="tt tasks"

function tt_update () {
    if (( $# == 0 )) then
        echo "wrong parameters"
        return 1
    fi
    if [[ -z $2 ]]; then
        date_args=()
    else
        date_args=(-d "$2")
    fi
    if ! date_str=$(date -Iminutes "${date_args[@]}"); then
        echo "wrong date format: $2"
        return 1
    fi
    printf "%s\t%s\n" "$date_str" "$1" >> "${TIMETRACK_FILE}"
}

function tts () {
    if [[ -z $1 ]]; then
        echo "wrong parameters"
        return 1
    fi
    task=$(ttt | grep -P "^$1\t" | cut -f3)
    if [[ -z $task ]]; then
        task=$1
    else
        echo "Continuing task \"${task}\""
    fi
    tt_update "$(printf "%s\t%s" "start" "$task")" "$2"
}

function ttoff () {
    tt_update off "$1"
}

function tton () {
    if [[ -z $1 ]]; then
        date_args=()
    else
        date_args=(-d "$1")
    fi
    if ! date_str=$(date "${date_args[@]}" +%A); then
        echo "wrong date format: $1"
        return 1
    fi
    printf "\n# %s\n" "$date_str" >> "${TIMETRACK_FILE}"

    tt_update on "$1"
}

function ttstop () {
    tt_update stop "$1"
}

function ttres () {
    tt_update resume "$1"
}

function ttrename () {
    if [[ -z $2 ]]; then
        event=$(printf "rename\t%s" "$1")
    else
        event=$(printf "rename\t%s\t%s" "$1" "$2")
    fi
    tt_update "$event"
}

function ttcont () {
    last=$(tt last-active)
    if [[ -n $last ]]; then
        echo "Continuing task \"${last}\""
        tts "$last" "$1"
    else
        echo "No previous active task, resuming work"
        ttres "$1"
    fi
}
