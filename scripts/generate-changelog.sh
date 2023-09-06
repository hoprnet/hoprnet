#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

milestone_number=${1}



# Decode the entry of a changelog
jq_decode() {
    echo ${1} | base64 --decode
}

declare -a section_feature section_bugs section_documentation

section_feature="### 🚀 New Features\n\n"
section_bugs="### 🐛 Bug\n\n"
section_other="### ⚡ Other\n\n"

# Adds an entry into the ChangeLog
add_entry_type() {
    id=${1}
    title=${2}
    labels=${3}
    if [[ ${labels} == *"feature"* ]]
    then
        section_feature="${section_feature} - #${id} - ${title}\n"
    elif [[ ${labels} == *"bug"* ]]
    then
        section_bugs="${section_bugs} - #${id} - ${title}\n"
    else
        section_other="${section_other} - #${id} - ${title}\n"
    fi
}



process_entries() {
    entries=${1}
    for item_encoded in ${entries}; do
        item_decoded=$(jq_decode ${item_encoded})
        id=$(echo "${item_decoded}" | jq -r '.number')
        title=$(echo "${item_decoded}" | jq -r '.title')
        labels=$(echo "${item_decoded}" | jq -r '.labels[].name' | tr '\n' ',' | sed 's/,$//')
        add_entry_type "${id}" "${title}" "${labels}"
    done
}

build_change_log() {
    change_log_content="Below there is a list with the contents of this release\n\n"
    if [[ ${section_feature} == *"-"* ]]
    then
        change_log_content="${change_log_content} ${section_feature}\n"
    fi
    if [[ ${section_bugs} == *"-"* ]]
    then
        change_log_content="${change_log_content} ${section_bugs}\n"
    fi
    if [[ ${section_other} == *"-"* ]]
    then
        change_log_content="${change_log_content} ${section_other}\n"
    fi
    echo -e ${change_log_content}
}


# Process Issues
issues=$(gh issue list --milestone ${milestone_number} --state all --json number,title,labels | jq -r 'to_entries[] | .value | @base64')
process_entries "$issues"

# Process PR
prs=$(gh pr list --state all --json number,title,labels,milestone | jq -r --argjson milestone_number ${milestone_number} 'to_entries[] | select(.value.milestone) | select(.value.milestone.number == $milestone_number).value | @base64')
process_entries "$prs"
build_change_log


