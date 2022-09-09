#!/bin/bash

# Takes cargo bench --bench benches -- --noplot --baseline master as input
# Formats it and posts results to PR on a GitHub as a comment

set -eu
set -o pipefail

echo $CI_COMMIT_BRANCH
RAW_REPORT=$1
PR_COMMENTS_URL="https://api.github.com/repos/paritytech/wasmi/issues/${CI_COMMIT_BRANCH}/comments"

sed -e '1,3d' \
    -e '/^B.*:.*$/d' \
    -e 's/^B[a-z]\+\s/\*\*/g' \
    -e 's/^\*\*.*$/&\*\*/g' \
    -e 's/^.*time/time/g' \
    -e 's/^[ \t]*//' \
    -e 's/^$/||/g' \
    -e 's/Performance has improved./:green_circle: **Performance has improved.**/g' \
    -e 's/Performance has regressed./:red_circle: **Performance has regressed.**/g' \
    -e 's/Change within noise threshold./:white_circle: **Change within noise threshold.**/g' \
    -e 's/No change in performance detected./:white_circle: **No change in performance detected.**/g' \
    -e 's/$/\\n/g' $1 \
    | tr -d '\n' \
    | tee formatted-report.txt

RESULT=$(cat formatted-report.txt)

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
    -d $"{ \"body\": \"| Benchmarks results |\n|---|\n ${RESULT} \" }"
