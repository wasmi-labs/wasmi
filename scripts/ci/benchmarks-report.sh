#!/bin/bash

# Takes raw reports from
# "cargo bench --bench benches -- --noplot --save-baseline master" output as 1st argument
# "cargo bench --bench benches -- --noplot --baseline master" output as 2nd argument
# Parses them to json and posts formatted results to PR on a GitHub as a comment
set -eu
set -o pipefail

PR_COMMENTS_URL="https://api.github.com/repos/paritytech/wasmi/issues/${CI_COMMIT_BRANCH}/comments"

# master report to json
echo "PARSING MASTER REPORT"
sed -e 's/^Found.*//g' \
    -e 's/^\s\+[[:digit:]].*$//g' \
    -e 's/\//_/g' \
    -e 's/^[a-z0-9_]\+/"&": {/g' \
    -e 's/time:\s\+\[.\{10\}/"time": "/g' \
    -e 's/.\{10\}\]/"},/g' \
    -e '1s/^/{\n/g' \
    -e '/^$/d' \
    -e 's/  */ /g' \
    -e 's/^ *\(.*\) *$/\1/' $1 \
    | sed -z 's/.$//' \
    | sed -e '$s/.$/}/g' \
    | tee target/criterion/output_master.json

# PR report to json
sed -e 's/^Found.*//g' \
    -e 's/^\s\+[[:digit:]].*//g' \
    -e 's/\//_/g' \
    -e 's/^[a-z0-9_]\+/"&": {/g' \
    -e 's/time:\s\+\[.\{10\}/"time": "/g' \
    -e 's/.\{10\}\]$/",/g' \
    -e 's/change:\s.\{10\}/"change":"/g' \
    -e 's/\s[-+].*$/",/g' \
    -e 's/\(No\|Ch\).*$/"perf_change":":white_circle:"},/' \
    -e 's/Performance has regressed./"perf_change":":red_circle:"},/' \
    -e 's/Performance has improved./"perf_change":":green_circle:"},/' \
    -e '1s/^/{\n/g' \
    -e '/^$/d' \
    -e 's/  */ /g' \
    -e 's/^ *\(.*\) *$/\1/' $2 \
    | sed -z 's/.$//' \
    | sed -e '$s/.$/}/g' \
    | tee target/criterion/output_pr.json

cd target/criterion

# Prepare report table
for d in */; do
    d=${d::-1}
    echo -n "| \`${d}\` "\
         "| $(cat output_master.json | jq .${d}.time | tr -d '"') "\
         "| $(cat output_pr.json | jq .${d}.time | tr -d '"') "\
         "| $(cat output_pr.json | jq .${d}.perf_change | tr -d '"') "\
         "$(cat output_pr.json | jq .${d}.change | tr -d '"') |\n" >> bench-final-report.txt
done

RESULT=$(cat bench-final-report.txt)

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
curl -X ${REQUEST_TYPE} ${PR_COMMENTS_URL} -v \
    -H "Cookie: logged_in=no" \
    -H "Authorization: token ${GITHUB_PR_TOKEN}" \
    -H "Content-Type: application/json; charset=utf-8" \
    -d $"{ \
\"body\": \
\"## CRITERION BENCHMARKS ## \n\n \
|BENCHMARK|MASTER|PR|Diff|\n \
|---|---|---|---|\n \
${RESULT}\n\n \
[Link to pipeline](${CI_JOB_URL}) \" \
}"
