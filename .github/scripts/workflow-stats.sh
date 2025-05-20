#!/bin/bash
set -euo pipefail
# Check if the required parameters are provided
if [ "$#" -ne 3 ]; then
  echo "Usage: $0 <workflow_file_name> <job_name> <workflow_runs>"
  echo "Example: $0 tests.yaml tests-units 1000"
  exit 1
fi

WORKFLOW_FILE=$1
JOB_NAME=$2
WORKFLOW_RUNS=$3
CACHE_DIR="./results"
mkdir -p "$CACHE_DIR"

# Temporary file to store workflow data
WORKFLOW_DATA="workflow_data.json"

# Fetch the first N workflow runs for the specified workflow file
echo "Fetching workflow runs for $WORKFLOW_FILE..."
gh run list -w "$WORKFLOW_FILE" --json databaseId,createdAt,conclusion,status --limit "$WORKFLOW_RUNS" >"$WORKFLOW_DATA"

# Initialize arrays to store waiting times and duration times
waiting_times=()
duration_times=()
success_count=0
total_count=0

# Iterate over each workflow run
echo "Processing workflow runs..."
for workflow_id in $(jq -r '.[] | .databaseId' "$WORKFLOW_DATA"); do
  # Define the cache file path
  CACHE_FILE="$CACHE_DIR/$workflow_id.json"

  # Check if the cache file exists
  if [ -f "$CACHE_FILE" ]; then
    # Load workflow details from the cache
    WORKFLOW_DETAILS=$(cat "$CACHE_FILE")
    #echo "Loaded cached data for Workflow ID: $workflow_id"
  else
    # Fetch detailed information for the workflow run and save it to the cache
    WORKFLOW_DETAILS=$(gh run view "$workflow_id" --json jobs,createdAt)
    echo "$WORKFLOW_DETAILS" >"$CACHE_FILE"
    echo "Fetched and cached data for Workflow ID: $workflow_id"
  fi

  # Extract the workflow-level createdAt timestamp
  workflow_created_at=$(echo "$WORKFLOW_DETAILS" | jq -r '.createdAt | fromdateiso8601 // empty')
  workflow_conclusion=$(echo "$WORKFLOW_DETAILS" | jq -r --arg JOB_NAME "$JOB_NAME" '.jobs[] | select(.name == $JOB_NAME) | .conclusion // empty')

  # Check if the specified job exists and succeeded
  JOB=$(echo "$WORKFLOW_DETAILS" | jq -r --arg JOB_NAME "$JOB_NAME" '.jobs[] | select(.name == $JOB_NAME and .conclusion == "success")')
  if [ -n "$JOB" ]; then
    # Extract job-level timestamps
    job_started_at=$(echo "$JOB" | jq -r '.startedAt | fromdateiso8601 // empty')
    job_updated_at=$(echo "$JOB" | jq -r '.completedAt | fromdateiso8601 // empty')

    # Skip if any timestamp is missing
    if [ -z "$workflow_created_at" ] || [ -z "$job_started_at" ] || [ -z "$job_updated_at" ]; then
      echo "Skipping workflow ID $workflow_id due to missing timestamps."
      continue
    fi

    # Debugging
    #echo "Processing workflowId: ${workflow_id}"

    # Calculate waiting time and duration time
    waiting_time=$((job_started_at - workflow_created_at))
    duration_time=$((job_updated_at - job_started_at))

    # Add times to arrays
    waiting_times+=("$waiting_time")
    duration_times+=("$duration_time")

    # Increment success count
    success_count=$((success_count + 1))
  fi

  # Increment total count
  #echo "Workflow ${workflow_id} conclusion: $workflow_conclusion"
  # Increment total count for success and failure
  if [[ $workflow_conclusion == "failure" ]] || [[ $workflow_conclusion == "success" ]]; then
    total_count=$((total_count + 1))
  fi
done

# Function to calculate statistics with pruning
calculate_stats() {
  local array_name=$1
  local times=($(eval echo "\${$array_name[@]}"))
  local count=${#times[@]}
  if [ "$count" -eq 0 ]; then
    echo "min: 0, max: 0, mean: 0, count: 0"
    return
  fi

  # Sort the array
  IFS=$'\n' sorted_times=($(sort -n <<<"${times[*]}"))
  unset IFS

  # Prune the lowest 5 and highest 5 values
  if [ "$count" -le 10 ]; then
    pruned_times=("${sorted_times[@]}")
  else
    pruned_times=("${sorted_times[@]:5:${#sorted_times[@]}-10}")
  fi

  # Recalculate count after pruning
  local pruned_count=${#pruned_times[@]}
  if [ "$pruned_count" -eq 0 ]; then
    echo "min: 0, max: 0, mean: 0, count: 0"
    return
  fi

  # Calculate min, max, and mean
  local min=${pruned_times[0]}
  local max=${pruned_times[$((pruned_count - 1))]}
  local sum=0

  for time in "${pruned_times[@]}"; do
    sum=$((sum + time))
  done

  local mean=$((sum / pruned_count))
  echo "min: $min, max: $max, mean: $mean, count: $count"
}

# Output results
echo "Workflow File: $WORKFLOW_FILE"
echo "Job Name: $JOB_NAME"
echo -n "Waiting Time: "
calculate_stats waiting_times

echo -n "Duration Time: "
calculate_stats duration_times

# Calculate success ratio
if [ "$total_count" -gt 0 ]; then
  success_ratio=$(echo "scale=2; $success_count * 100 / $total_count" | bc)
else
  success_ratio=0
fi
echo "Success Ratio: $success_ratio %"
