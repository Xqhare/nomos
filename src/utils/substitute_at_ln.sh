#!/usr/bin/env bash

# Usage: ./substitute_at_ln.sh <lnr> <new> <file_path>
# Adapted from: https://www.linuxquestions.org/questions/linux-newbie-8/bash-replace-text-on-a-specific-line-4175684442/

set -e

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <lnr> <new_text> <file_path>"
    exit 1
fi

subst_lnr(){
  local lnr=$1 repl=$2
  local line nr=0
  while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ $((++nr)) -eq $lnr ]]; then
      line=$repl
    fi
    printf "%s\n" "$line"
  done
}

target_ln = $1
new_text = $2
file_path = $3

tmp_file = $(mktemp)

subst_lnr "$target_ln" "$new_text" < "$file_path" > "$tmp_file"
mv "$tmp_file" "$file_path"

exit 0
