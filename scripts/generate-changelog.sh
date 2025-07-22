#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

current_version=${1}
milestone_number=$(gh api repos/:owner/:repo/milestones | jq -r --arg version "${current_version}" ' to_entries[] | select(.value.title == $version).value.number')
if [ -z "${milestone_number}" ]; then
  echo "[ERROR] No milestone found for version ${current_version}" >&2
  exit 1
fi
changelog_format=${2:-github}
include_open=${3:-false}

# Decode the entry of a changelog
jq_decode() {
  echo "${1}" | base64 --decode
}

# Process entries and build the changelog array
process_entries() {
  # Sanitize and validate entries to ensure proper JSON formatting
  entries=${1}
  for item_encoded in ${entries}; do
    item_decoded=$(jq_decode "${item_encoded}")
    # Validate JSON format
    if ! echo "${item_decoded}" | jq empty; then
      echo "[ERROR] Invalid JSON record: ${item_decoded}" >>/dev/stderr
      exit 1
    fi
    id=$(echo "${item_decoded}" | jq -r '.number')
    title=$(echo "${item_decoded}" | jq -r '.title')
    labels=$(echo "${item_decoded}" | jq -r '.labels[]?.name' | tr '\n' ',' | sed 's/,$//')
    state=$(echo "${item_decoded}" | jq -r '.state' | tr '[:upper:]' '[:lower:]')
    author=$(echo "${item_decoded}" | jq -r '.author.login')
    date=$(echo "${item_decoded}" | jq -r '.closedAt // empty' | awk -F 'T' '{print $1}')
    if [[ -z ${date} ]]; then
      date=$(date '+%Y-%m-%d') # Fallback to current date
    fi
    if [[ $state == "open" ]] && [[ $include_open == false ]]; then
      echo "[ERROR] Error generating changelog from a milestone with open items" >>/dev/stderr
      exit 1
    fi
    # Extract changelog_type and component from the title
    changelog_type=$(echo "${title}" | awk -F ':' '{print $1}' | awk -F '(' '{print $1}')
    component=$(echo "${title}" | awk -F ':' '{print $1}' | awk -F '(' '{print $2}')
    component=${component%)} # Remove trailing parenthesis if present
    # Add fallback for changelog_type and component if they are empty
    changelog_type=${changelog_type:-"unknown"}
    component=${component:-"general"}
    #echo "[DEBUG] Processing entry: id=${id}, title=${title}, author=${author}, labels=${labels}, state=${state}, date=${date}, changelog_type=${changelog_type}, component=${component}" >>/dev/stderr
    changelog_entries+=("$(jq -nc --arg id "$id" \
      --arg title "$title" \
      --arg author "$author" \
      --arg labels "$labels" \
      --arg state "$state" \
      --arg date "$date" \
      --arg ctype "$changelog_type" \
      --arg comp "$component" \
      '{id:$id,title:$title,author:$author,labels:$labels,state:$state,date:$date,changelog_type:$ctype,component:$comp}')")
  done
}

# Build the changelog in GitHub format
github_format_changelog() {
  section_feature="\n### 🚀 New Features\n\n"
  section_fix="\n### 🐞 Fixes\n\n"
  section_refactor="\n### 🧹 Refactor\n\n"
  section_ci="\n### ⚙️ Automation\n\n"
  section_documentation="\n### 📚 Documentation\n\n"
  section_performance="\n### ⚡ Performance Improvements\n\n"
  section_other="\n### 🌟 Other\n\n"
  local change_log_content="What's changed\n"

  for entry in "${changelog_entries[@]}"; do
    id=$(echo "$entry" | jq -r '.id')
    title=$(echo "$entry" | jq -r '.title')
    author=$(echo "$entry" | jq -r '.author')
    case ${title} in
    "feat("* | "feat:"* | "chore("* | "chore:"*)
      section_feature+="* ${title} by @${author} in #${id}\n"
      ;;
    "fix("* | "fix:"*)
      section_fix+="* ${title} by @${author} in #${id}\n"
      ;;
    "refactor("* | "refactor:"* | "style("* | "style:"*)
      section_refactor+="* ${title} by @${author} in #${id}\n"
      ;;
    "docs("* | "docs:"*)
      section_documentation+="* ${title} by @${author} in #${id}\n"
      ;;
    "perf("* | "perf:"*)
      section_performance+="* ${title} by @${author} in #${id}\n"
      ;;
    "test("* | "test:"* | "build("* | "build:"*)
      section_ci+="* ${title} by @${author} in #${id}\n"
      ;;
    *)
      section_other+="* ${title} by @${author} in #${id}\n"
      ;;
    esac
  done

  # The exclamation mark (!) in ${!section} is used for indirect variable expansion in Bash. It allows you to reference the value of a variable whose name is stored in another variable.
  for section in section_feature section_fix section_refactor section_ci section_documentation section_performance section_other; do
    if [[ ${!section} == *" by "* ]]; then
      change_log_content+="${!section}\n"
    fi
  done

  echo -e "${change_log_content}"
}

# Build the changelog in JSON format
json_format_changelog() {
  local change_log_content="$(printf '%s\n' "${changelog_entries[@]}" | jq -s -c '.')"
  echo -e "${change_log_content}"
}

# Function to determine the release type
get_release_type() {

  # Default to stable
  local release_type="stable"

  # Check for experimental, breaking, or release labels
  if [[ $(printf '%s\n' "${changelog_entries[@]}" | jq -r '.labels' | grep -E "experimental|breaking") ]]; then
    release_type="unstable"
  fi

  # Check if the version contains "-rc." or is the first release (x.y.0)
  if [[ ${current_version} == *"-rc."* ]] || [[ ${current_version} =~ ^[0-9]+\.[0-9]+\.0$ ]]; then
    release_type="unstable"
  fi

  echo "${release_type}"
}

# Function to determine the urgency level
get_urgency_level() {
  local version=${1}

  # Extract the patch number from the version
  local patch_number=$(echo ${version} | awk -F '.' '{print $3}' | awk -F '-' '{print $1}')

  # Determine urgency based on patch number
  if [[ ${version} == *"-rc."* ]] || [[ ${patch_number} -eq 0 ]]; then
    echo "optional"
  else
    echo "medium"
  fi
}

# Build the changelog in Debian format
debian_format_changelog() {

  local distribution=$(get_release_type)
  local urgency=$(get_urgency_level "${current_version}")
  local maintainer="Hoprnet <tech@hoprnet.org>"
  local date="$(date -R)"

  # Ensure clean assignment to debian_changelog
  local debian_changelog="hoprd (${current_version}) ${distribution}; urgency=${urgency}\n"

  for entry in "${changelog_entries[@]}"; do
    id=$(echo "$entry" | jq -r '.id')
    title=$(echo "$entry" | jq -r '.title')
    author=$(echo "$entry" | jq -r '.author')
    # Check the length of the entry line and adjust if necessary
    entry_line="  * ${title} by @${author} in #${id}\n"
    if [[ ${#entry_line} -le 80 ]]; then
      debian_changelog+="${entry_line}"
    else
      # Calculate the maximum length for the title to fit within 80 characters
      max_title_length=$((80 - 2 - ${#entry_line} + ${#title}))
      if ((max_title_length < 1)); then
        max_title_length=1
      fi
      truncated_title=$(echo "${title}" | cut -c 1-${max_title_length})
      debian_changelog+="  * ${truncated_title}... by @${author} in #${id}\n"
    fi
  done

  debian_changelog+="\n -- ${maintainer}  ${date}\n"

  # Print the changelog
  echo -e "${debian_changelog}"
}

# Build the changelog in RPM format
rpm_format_changelog() {
  local maintainer="Hoprnet <tech@hoprnet.org>"
  local rpm_changelog=""

  # Sort entries by date and then by author
  sorted_entries=$(printf '%s\n' "${changelog_entries[@]}" | jq -s 'sort_by([.date, .author]) | reverse')

  # Group entries by date and author, and build the changelog
  local current_date=""
  local current_author=""
  while read -r entry; do
    id=$(echo "$entry" | jq -r '.id')
    title=$(echo "$entry" | jq -r '.title')
    author=$(echo "$entry" | jq -r '.author')
    date=$(echo "$entry" | jq -r '.date')
    changelog_type=$(echo "$entry" | jq -r '.changelog_type')
    component=$(echo "$entry" | jq -r '.component')

    if [[ $date != "$current_date" || $author != "$current_author" ]]; then
      current_date="$date"
      current_author="$author"
      rpm_changelog+="* ${date} ${author} - ${current_version}\n"
    fi

    # Remove the prefix "<changelog_type>(component): " from the title if present
    clean_title=$(echo "$title" | sed -E 's/^.*\): //')

    # # Add fallback for clean_title if it is empty
    # clean_title=${clean_title:-"Untitled"}

    rpm_changelog+="- [${changelog_type}][${component}] ${clean_title} in #${id}\n"
  done <<<"$(echo "${sorted_entries}" | jq -c '.[]')"

  # Print the changelog
  echo -e "${rpm_changelog}"
}

# Initialize the changelog entries array
declare -a changelog_entries

# Process Issues
issues=$(gh issue list --milestone ${milestone_number} --state all --limit 300 --json number,title,labels,state,author,closedAt | jq -r 'to_entries[] | .value | @base64')
process_entries "$issues"

# Process PR
prs=$(gh pr list --state all --limit 300 --json number,title,labels,milestone,state,author,closedAt | jq -r --argjson milestone_number ${milestone_number} 'to_entries[] | select(.value.milestone) | select(.value.milestone.number == $milestone_number).value | @base64')
process_entries "$prs"

# Print the changelog in the requested format
case $changelog_format in
github)
  github_format_changelog
  ;;
json)
  json_format_changelog
  ;;
deb | archlinux) # archlinux does not have a specific format, we will use the debian format
  debian_format_changelog
  ;;
rpm)
  rpm_format_changelog
  ;;
all)
  json_format_changelog
  github_format_changelog
  debian_format_changelog
  rpm_format_changelog
  ;;
*)
  echo "Unsupported format: ${changelog_format}" >>/dev/stderr
  exit 1
  ;;
esac
