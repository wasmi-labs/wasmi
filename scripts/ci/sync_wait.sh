#!/bin/bash
# Script waits until file benchmarks/criterion/output-criterion.txt
# appears

for i in $(seq 1 600); do
  git pull -q origin gh-pages
  sleep 1
  ls benchmarks/criterion/output-criterion.txt 2>/dev/null
  if [ $? = 0 ]; then
    break;
  fi
  echo "Waiting..."
done
