#!/bin/bash
set -e

fun_jobs(){
  job_list="$1"
  job_skip="skip"
  echo "$job_list" | sed "s/\[//g" | sed "s/\]//g" | sed "s/,/\n/g" > job_run.txt
  while read -r LINE;
  do
    LINE="$(echo "$LINE" | awk '{gsub(/^\s+|\s+$/,"");print}')"
    echo "LINE is "$LINE
    if [[ -n $LINE ]] && [[ $GITHUB_WORKFLOW == "$LINE"* ]];then
      echo "job_name is"$LINE
      echo $GITHUB_WORKFLOW
      job_skip="run"
    fi
  done < "job_run.txt"
  echo "JOB skip is" $job_skip
  echo "::set-output name=job_skip::$job_skip"
}

fun_pasing_message(){
  set +e
  MESSAGE=$1
  #pass job run list
  echo "$MESSAGE" | grep -q "ci-runs-only:"
  if [ $? -eq 0 ]; then
    job_run_list=`echo "${MESSAGE}"| grep "ci-runs-only" | awk -F ':' '{print $2}'`
  else
    job_run_list=" [ Chaos CI,Cargo Clippy,Coverage Test,E2E Tests,Conventional PR,Unit Tests] "
  fi
  echo "job_run_list is ""$job_run_list"
  
  set -e
  #set reqiured output
  fun_jobs "$job_run_list"
}

if [[ $GITHUB_EVENT_NAME == "pull_request" ]];then
  MESSAGE="$PR_COMMONS_BODY"
  fun_pasing_message "$MESSAGE"
fi
if [[ $GITHUB_EVENT_NAME == "push" ]];then
  job_run_list=" [ E2E Tests] "
  # job_run_list=" [ Chaos CI,Cargo Clippy,Coverage Test,E2E Tests,Conventional PR,Unit Tests] "
  #set reqiured output
  fun_jobs "$job_run_list"
fi
