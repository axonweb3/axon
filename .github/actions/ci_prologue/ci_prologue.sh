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
  job_run_list=" [ Conventional PR"
  chaos_ci_box_value=`echo "${MESSAGE}" |grep 'Chaos CI' |awk '{print $2}'`
  cargo_clippy_box_value=`echo "${MESSAGE}" |grep 'Cargo Clippy' |awk '{print $2}'`
  coverage_test_box_value=`echo "${MESSAGE}" |grep 'Coverage Test' |awk '{print $2}'`
  E2E_tests_box_value=`echo "${MESSAGE}" |grep 'E2E Tests' |awk '{print $2}'`
  code_format_box_value=`echo "${MESSAGE}" |grep 'Code Format' |awk '{print $2}'`
  OCT_1_to_5_and_12_to_15_box_value=`echo "${MESSAGE}" |grep 'OCT 1-5 And 12-15' |awk '{print $2}'`
  OCT_6_to_10_box_value=`echo "${MESSAGE}" |grep 'OCT 6-10' |awk '{print $2}'`
  OCT_11_box_value=`echo "${MESSAGE}" |grep 'OCT 11' |awk '{print $2}'`
  OCT_16_to_19_box_value=`echo "${MESSAGE}" |grep 'OCT 16-19' |awk '{print $2}'`
  unit_tests_box_value=`echo "${MESSAGE}" |grep 'Unit Tests' |awk '{print $2}'`
  v3_core_tests_box_value=`echo "${MESSAGE}" |grep 'v3 Core Tests' |awk '{print $2}'`
  web3_compatible_tests_box_value=`echo "${MESSAGE}" |grep 'Web3 Compatible Tests' |awk '{print $2}'`
  if [[ $chaos_ci_box_value == '[x]' ]];then
         job_run_list=$job_run_list",Chaos CI"
  fi
  if [[ $cargo_clippy_box_value == '[x]' ]];then
         job_run_list=$job_run_list",Cargo Clippy"
  fi
  if [[ $coverage_test_box_value == '[x]' ]];then
         job_run_list=$job_run_list",Coverage Test"
  fi
  if [[ $code_format_box_value == '[x]' ]];then
         job_run_list=$job_run_list",code_format_box_value"
  fi
  if [[ $OCT_1_to_5_and_12_to_15_box_value == '[x]' ]];then
         job_run_list=$job_run_list",OCT 1-5 And 12-15"
  fi
  if [[ $OCT_6_to_10_box_value == '[x]' ]];then
         job_run_list=$job_run_list",OCT 16-19"
  fi
  if [[ $unit_tests_box_value == '[x]' ]];then
         job_run_list=$job_run_list",Unit Tests"
  fi
  if [[ $v3_core_tests_box_value == '[x]' ]];then
         job_run_list=$job_run_list",v3 Core Tests"
  fi
  if [[ $web3_compatible_tests_box_value == '[x]' ]];then
         job_run_list=$job_run_list",Web3 Compatible Tests"
  fi
  job_run_list=$job_run_list" ]"

  #pass job run list
  echo "$MESSAGE" | grep -q "ci-runs-only:"
  if [ $? -eq 0 ]; then
    job_run_list=`echo "${MESSAGE}"| grep "ci-runs-only" | awk -F ':' '{print $2}'`
  else
    job_run_list=" [ Code Format,Chaos CI,Cargo Clippy,Coverage Test,E2E Tests,Conventional PR,Unit Tests,Web3 Compatible Tests,OCT 1-5 And 12-15,OCT 6-10,OCT 11,OCT 16-19,v3 Core Tests] "
  fi
  echo "job_run_list is ""$job_run_list"
  
  set -e
  #set reqiured output
  fun_jobs "$job_run_list"
}

if [[ $GITHUB_EVENT_NAME == "pull_request" ]];then
  echo "$PR_AUTHOR"
  
  if [[ $PR_AUTHOR == "dependabot[bot]" ]]; then
    # Only run below jobs when pr suthor is dependabot.
    job_run_list=" [ Cargo Clippy,Code Format,E2E Tests,Unit Tests ] "
    fun_jobs "$job_run_list"
  else
    MESSAGE="$PR_COMMONS_BODY"
    fun_pasing_message "$MESSAGE"
  fi
fi
if [[ $GITHUB_EVENT_NAME == "push" ]];then
  job_run_list=" [ Code Format,Chaos CI,Cargo Clippy,Coverage Test,E2E Tests,Conventional PR,Unit Tests,Web3 Compatible Tests,OCT 1-5 And 12-15,OCT 6-10,OCT 11,OCT 16-19,v3 Core Tests] "
  #set reqiured output
  fun_jobs "$job_run_list"
fi
