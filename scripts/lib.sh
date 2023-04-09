#!/bin/bash
set -euo pipefail
cd "$( dirname "${BASH_SOURCE[0]}" )/..";

banner_min_width=80

function info {
    echo "[INFO] $@"
}

function error {
    echo "[ERROR] $@"
}

function warn {
    echo "[WARN] $@"
}

function read_until_set {
  local __v=${1}
  local message="${2}"
  local silent=${3:-"false"}
  
  if [ -z "${!__v:-}" ]
  then
    local v=""
    while [ -z "$v" ]; do
      printf "%s: " "$message"
      read -r v
    done
    eval $__v="'$v'"
  fi
  if [ "${silent}" != "true" ]; then
    printf "%s: '%s'\n" ${1} ${!1}
  fi
}

function read_optionally {
  local __v=${1}
  local message="${2}"
  local silent=${3:-"false"}
  
  if [ -z "${!__v:-}" ]
  then
    local v=""
    
    printf "%s: " "$message"
    read -r -e v
    
    if [ -z "${v:-}" ] && [ ! -z "${4:-}" ]
    then
      v="${3}" 
    fi
    eval $__v="'$v'"
  fi
  if [ "${silent}" != "true" ]; then
    printf "%s: '%s'\n" ${1} ${!1}
  fi
}

function check_prerequisite {
  if ! command -v ${1} &> /dev/null
  then
    error "COMMAND '${1}' could not be found! Please install first!"
    exit 1
  fi
}

function get_user_confirmation {

    # Pass if running unattended
    [[ "${OPT_UNATTENDED:-false}" = true ]] && return 0

    # Fail if STDIN is not a terminal (there's no user to confirm anything)
    [[ -t 0 ]] || return 1

    # Otherwise, ask the user
    #
    msg=$([ -n "${1:-}" ] && echo -n "$1" || echo -n "Continue? (y/n) ")
    yes=$([ -n "${2:-}" ] && echo -n "$2" || echo -n "y")
    echo -n "$msg"
    read c && [ "$c" = "$yes" ] && return 0
    return 1
}

function parse_and_run_command() {
  if [[ $# = 0 ]]; then
    sub_default
    exit 0
  fi 

  while [[ $# -gt 0 ]]
  do
  arg="$1"
  case $arg in
      "" | "-h" | "--help")
          sub_default
          shift
          ;;
      --debug)
          set -x
          DEBUG=true
          shift
          ;;
      *)
          shift
          sub_${arg} $@
          exit_code=$?
          if [ $exit_code = 127 ]; then
              echo "Error: '$arg' is not a known subcommand." >&2
              echo "       Run '$progname --help' for a list of known subcommands." >&2
              exit 1
          elif [ $exit_code = 0 ]; then
              exit 0
          else
              exit $exit_code
          fi
          ;;
  esac
  done
}

function print_generic_options() {
  echo "Generic Options:"
  echo "    --debug                     print out every command running"
}

function deactivate_debug() {
  set +x
}

function activate_debug() {
  # don't do it if not in debug-mode
  if [ -n "${DEBUG+x}" ]; then
    set -x
  fi
}
