#!/bin/bash

# This script prepares table with results of benchmarks and posts it to the
# GitHub's PR as an issue comment.

set -eu
set -o pipefail

# Transform timing details into more readable format
function format_time {
    if (( $(echo $1'<'1000 | bc -l) ))
      then printf "%.2fns" $1
    elif (( $(echo $1'<'1000000 | bc -l) ))
      then printf "%.2fµs" $(echo $1/1000 | bc -l )
    else
      printf "%.2fms" $(echo $1/1000000 | bc -l )
    fi
}

# Derive performance change status from benchmarks raw command prompt log
function get_performance_change_status {
    if echo $1 | grep -e "Performance has regressed" >> /dev/null
      then echo ":red_circle:"
    elif echo $1 | grep -e "Performance has improved" >> /dev/null
      then echo ":green_circle:"
    elif echo $1 | grep -e "No change in performance detected" >> /dev/null
      then echo ":white_circle:"
    elif echo $1 | grep -e "Change within noise threshold" >> /dev/null
      then echo ":white_circle:"
    fi
}

# Native vs wasmtime overhead status badges
function wasmtime_overhead_status {
    if [[ $1 -le 50 ]]
      then echo ":green_circle: $1%"
    elif [[ $1 -le 100 ]]
      then echo ":yellow_circle: $1%"
    else
      echo ":red_circle: $1%"
    fi
}

PR_COMMENTS_URL="https://api.github.com/repos/paritytech/wasmi/issues/${CI_COMMIT_BRANCH}/comments"

pushd ./target/ci/criterion

# Format benchmarks details into a table
RESULT=$(for d in */; do
            BENCH_ID=$(jq .full_id ${d}master/benchmark.json | tr -d '"' | sed -e 's/\//\/<\/tt><br><tt>/' )

            MASTER_TIME=$(jq .slope.point_estimate ${d}master/estimates.json)
            PR_TIME=$(jq .slope.point_estimate ${d}new/estimates.json)
            PERF_CHANGE=$(get_performance_change_status "$(grep -A 3 -e $(echo "${d::-1}" | tr "_" ".") ../bench-pr)")
            DIFF=$(jq .mean.point_estimate ${d}change/estimates.json)

            WASM_MASTER_TIME=$(jq .slope.point_estimate ../wasmtime-criterion/${d}master-wasm/estimates.json)
            WASM_PR_TIME=$(jq .slope.point_estimate ../wasmtime-criterion/${d}new/estimates.json)
            WASM_PERF_CHANGE=$(get_performance_change_status "$(grep -A 3 -e $(echo "${d::-1}" | tr "_" ".") ../wasmtime-pr)")
            WASM_DIFF=$(jq .mean.point_estimate ../wasmtime-criterion/${d}change/estimates.json)

            WT_OVERHEAD=$(echo "($WASM_PR_TIME-$PR_TIME)/$PR_TIME*100" | bc -l | xargs printf "%.0f")

            echo -n "<tr><td><tt>$BENCH_ID<\/td>"\
                "<td nowrap> $(format_time $MASTER_TIME)<\/td>" \
                "<td nowrap> $(format_time $PR_TIME)<\/td>" \
                "<td nowrap> $PERF_CHANGE $(echo $DIFF*100 | bc -l | xargs printf "%.2f%%")<\/td>" \
                "<td nowrap> $(format_time $WASM_MASTER_TIME)<\/td>" \
                "<td nowrap> $(format_time $WASM_PR_TIME)<\/td>" \
                "<td nowrap> $WASM_PERF_CHANGE $(echo $WASM_DIFF*100 | bc -l | xargs printf "%.2f%%")<\/td>" \
                "<td nowrap> $(wasmtime_overhead_status $WT_OVERHEAD)<\/td><\/tr>"
        done)

popd

# Check whether comment from paritytech-cicd-pr already exists
EXISTING_COMMENT_URL=$(curl --silent $PR_COMMENTS_URL \
                       | jq -r ".[] \
                       | select(.user.login == \"paritytech-cicd-pr\") \
                       | .url" \
                       | head -n1)

# If there is already a comment by the user `paritytech-cicd-pr` in the PR which triggered
# this run, then we can just edit this comment (using `PATCH` instead of `POST`).
REQUEST_TYPE="POST"
if [ ! -z "$EXISTING_COMMENT_URL" ]; then
   REQUEST_TYPE="PATCH";
   PR_COMMENTS_URL="$EXISTING_COMMENT_URL"
fi

echo "Comment will be posted here $PR_COMMENTS_URL"

# POST/PATCH comment to the PR
curl -X ${REQUEST_TYPE} ${PR_COMMENTS_URL} \
    -H "Cookie: logged_in=no" \
    -H "Authorization: token ${GITHUB_PR_TOKEN}" \
    -H "Content-Type: application/json; charset=utf-8" \
    -d $"{ \
\"body\": \
\"## BENCHMARKS ## \n\n \
<table> \
<thead> \
<tr> \
<th><\/th><th colspan=3>NATIVE<\/th><th colspan=3>WASMTIME<\/th><th></th> \
<\/tr> \
<tr> \
<th>BENCHMARK<\/th><th>MASTER<\/th><th>PR<\/th><th>DIFF<\/th><th>MASTER<\/th><th>PR<\/th><th>DIFF<\/th><th>WASMTIME OVERHEAD<\/th> \
<\/tr> \
<\/thead> \
<tbody> \
${RESULT} \
<\/tbody> \
<\/table> \n\n \
[Link to pipeline](${CI_JOB_URL}) \" \
}"
