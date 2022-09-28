#!/bin/bash

# This script takes as an argument benchmark report JSON file produced
# by the command "cargo criterion --message-format=json". Executed first
# against a 'master' branch and on a PR commit afterwards.
# Formats it using 'jq' with filters defined in the './scripts/ci/benchmark-filter.jq'.
# And posts formatted results to a PR on a GitHub as an issue comment.

set -eu
set -o pipefail

function format_time {
    if (( $(echo $1'<'1000 | bc -l) ))
      then
        printf "%\.2f ns" $1
    elif (( $(echo $1'<'1000000 | bc -l) ))
      then
        printf "%\.2f µs" $(echo $1/1000 | bc -l )
    else
        printf "%\.2f ms" $(echo $1/1000000 | bc -l )
    fi
}

PR_COMMENTS_URL="https://api.github.com/repos/paritytech/wasmi/issues/${CI_COMMIT_BRANCH}/comments"

pushd ./target/ci/criterion

# Format benchmarks into a table
RESULT=$(for d in */; do
            MASTER_TIME=$(jq .slope.point_estimate ${d}master/estimates.json)
            PR_TIME=$(jq .slope.point_estimate ${d}new/estimates.json)
            DIFF=$(jq .mean.point_estimate ${d}change/estimates.json)
            WASM_MASTER_TIME=$(jq .slope.point_estimate ../wasmtime-criterion/${d}master-wasm/estimates.json)
            WASM_PR_TIME=$(jq .slope.point_estimate ../wasmtime-criterion/${d}new/estimates.json)
            WASM_DIFF=$(jq .mean.point_estimate ../wasmtime-criterion/${d}change/estimates.json)

            echo -n "| \`${d::-1}\` "\
                "| $(format_time $MASTER_TIME)" \
                "| $(format_time $PR_TIME)" \
                "| $(echo $DIFF*100 | bc -l | xargs printf "%\.2f") %" \
                "| $(format_time $WASM_MASTER_TIME)" \
                "| $(format_time $WASM_PR_TIME)" \
                "| $(echo $WASM_DIFF*100 | bc -l | xargs printf "%\.2f") %|\n"
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
\"## CRITERION BENCHMARKS ## \n\n \
|BENCHMARK|MASTER|PR|DIFF|MASTER(WASM)|PR(WASM)|DIFF(WASM)|\n \
|---|---:|---:|---|---:|---:|---|\n \
${RESULT}\n\n \
[Link to pipeline](${CI_JOB_URL}) \" \
}"
