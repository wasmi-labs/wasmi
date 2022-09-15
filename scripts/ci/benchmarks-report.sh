#!/bin/bash

# Takes cargo bench --bench benches -- --noplot --baseline master as input
# Formats it and posts results to PR on a GitHub as a comment

set -eu
set -o pipefail

echo $CI_COMMIT_BRANCH
RAW_REPORT=$1
PR_COMMENTS_URL="https://api.github.com/repos/paritytech/wasmi/issues/${CI_COMMIT_BRANCH}/comments"

# master report to json
echo "PARSING MASTER REPORT"
sed -e 's/^Found.*//g' \
    -e 's/^\s\+[[:digit:]].*//g' \
    -e 's/\//_/g' \
    -e 's/^[a-z0-9_]\+/"&": {/g' \
    -e 's/time:\s\+\[.\{10\}/"time": "/g' \
    -e 's/.\{5\}\]$/"},/g' \
    -e '1s/^/{\n/g' \
    -e '/^$/d' \
    -e 's/  */ /g' -e 's/^ *\(.*\) *$/\1/' $1 \
    | sed -z 's/.$//' \
    | sed -e '$s/.$/}/g' \
    | tee target/criterion/output_master.json

echo -e "\nSHOW PARSED MASTER REPORT\n"
cat target/criterion/output_master.json

# PR report to json
echo -e "\nPARSING PR REPORT\n"

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
    -e 's/  */ /g' -e 's/^ *\(.*\) *$/\1/' $2 \
    | sed -z 's/.$//' \
    | sed -e '$s/.$/}/g' \
    | tee target/criterion/output_pr.json

echo -e "\nSHOW PARSED PR REPORT\n"

cat target/criterion/output_pr.json

cd target/criterion
echo

for d in */; do
    echo -e "GETTING BENCHMARK ${d::-1} DETAILS\n"
    echo -n "| ${d::-1} "\
         "| $(cat output_master.json | jq .${d::-1}.time | tr -d '"') "\
         "| $(cat output_pr.json | jq .${d::-1}.time | tr -d '"') "\
         "| $(cat output_pr.json | jq .${d::-1}.perf_change | tr -d '"') "\
         "$(cat output_pr.json | jq .${d::-1}.change | tr -d '"') |\n"
    echo -n "| ${d::-1} "\
         "| $(cat output_master.json | jq .${d::-1}.time | tr -d '"') "\
         "| $(cat output_pr.json | jq .${d::-1}.time | tr -d '"') "\
         "| $(cat output_pr.json | jq .${d::-1}.perf_change | tr -d '"') "\
         "$(cat output_pr.json | jq .${d::-1}.change | tr -d '"') |\n" >> bench-final-report.txt
done

RESULT=$(cat bench-final-report.txt)
echo -e "RESULT: \n $RESULT"

# If there is already a comment by the user `paritytech-cicd-pr` in the PR which triggered
# this run, then we can just edit this comment (using `PATCH` instead of `POST`).
EXISTING_COMMENT_URL=$(curl --silent $PR_COMMENTS_URL | \
  jq -r ".[] | select(.user.login == \"paritytech-cicd-pr\") | .url" | \
  head -n1
)
echo $EXISTING_COMMENT_URL

REQUEST_TYPE="POST"
if [ ! -z "$EXISTING_COMMENT_URL" ]; then
   REQUEST_TYPE="PATCH";
   PR_COMMENTS_URL="$EXISTING_COMMENT_URL"
fi

echo $REQUEST_TYPE
echo $PR_COMMENTS_URL

curl -X ${REQUEST_TYPE} ${PR_COMMENTS_URL} -v \
    -H "Cookie: logged_in=no" \
    -H "Authorization: token ${GITHUB_PR_TOKEN}" \
    -H "Content-Type: application/json; charset=utf-8" \
    -d $"{ \"body\": \"|BENCHMARK|MASTER|PR|Diff|\n|---|---|---|---|\n ${RESULT} \" }"
