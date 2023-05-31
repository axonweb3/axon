#!/usr/bin/env bash

# ENVIRONMENT VARIABLES:
#
#   * AWS_ACCESS_KEY, required, the AWS access key
#   * AWS_SECRET_KEY, required, the AWS secret key
#   * AWS_EC2_TYPE, optional, default is c6g.xlarge, the AWS EC2 type


set -euo pipefail

AWS_ACCESS_KEY=${AWS_ACCESS_KEY}
AWS_SECRET_KEY=${AWS_SECRET_KEY}
DOCKER_USER=${DOCKER_USER}
DOCKER_PASSWORD=${DOCKER_PASSWORD}
AWS_EC2_TYPE=${AWS_EC2_TYPE:-"c6g.xlarge"}
START_TIME=${START_TIME:-"$(date +%Y-%m-%d' '%H:%M:%S.%6N)"}
AXON_TAG=${AXON_TAG}
JOB_ID=${JOB_ID:-"build-arm4-$(date +'%Y-%m-%d')-in-10h"}
SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
JOB_DIRECTORY="$(dirname "$SCRIPT_PATH")/job/$JOB_ID"
ANSIBLE_DIRECTORY=$JOB_DIRECTORY/ansible
ANSIBLE_INVENTORY=$JOB_DIRECTORY/ansible/inventory.yml
TERRAFORM_DIRECTORY="$JOB_DIRECTORY/terraform"
SSH_PRIVATE_KEY_PATH=$JOB_DIRECTORY/ssh/id
SSH_PUBLIC_KEY_PATH=$JOB_DIRECTORY/ssh/id.pub

function job_setup() {
    mkdir -p $JOB_DIRECTORY
    cp -r "$(dirname "$SCRIPT_PATH")/ci/ansible"   $JOB_DIRECTORY/ansible
    cp -r "$(dirname "$SCRIPT_PATH")/ci/terraform" $JOB_DIRECTORY/terraform

    ssh_gen_key
    ansible_setup
}

function job_clean() {
    rm -rf $JOB_DIRECTORY
}

function ssh_gen_key() {
    # Pre-check whether "./ssh" existed
    if [ -e "$SSH_PRIVATE_KEY_PATH" ]; then
        echo "Info: $SSH_PRIVATE_KEY_PATH already existed, reuse it"
        return 0
    fi

    mkdir -p "$(dirname $SSH_PRIVATE_KEY_PATH)"
    ssh-keygen -t rsa -N "" -f $SSH_PRIVATE_KEY_PATH
}

function terraform_config() {
    export TF_VAR_access_key=$AWS_ACCESS_KEY
    export TF_VAR_secret_key=$AWS_SECRET_KEY
    export TF_VAR_prefix=$JOB_ID
    export TF_VAR_private_key_path=$SSH_PRIVATE_KEY_PATH
    export TF_VAR_public_key_path=$SSH_PUBLIC_KEY_PATH
}

# Allocate AWS resources defined in Terraform.
#
# The Terraform directory is "./terraform".
function terraform_apply() {
    terraform_config

    cd $TERRAFORM_DIRECTORY
    terraform init
    terraform plan
    terraform apply -auto-approve
    terraform output | grep -v EOT | tee $ANSIBLE_INVENTORY
}

# Destroy AWS resources
function terraform_destroy() {
    terraform_config

    cd $TERRAFORM_DIRECTORY
    terraform destroy -auto-approve
}

function ansible_config() {
    export ANSIBLE_PRIVATE_KEY_FILE=$SSH_PRIVATE_KEY_PATH
    export ANSIBLE_INVENTORY=$ANSIBLE_INVENTORY
}

# Setup Ansible running environment.
function ansible_setup() {
    cd $ANSIBLE_DIRECTORY
    echo "image_tag: $AXON_TAG"> $JOB_DIRECTORY/ansible/config.yml
    echo "docker_user: $DOCKER_USER">> $JOB_DIRECTORY/ansible/config.yml
    echo "docker_password: $DOCKER_PASSWORD">> $JOB_DIRECTORY/ansible/config.yml
}

# Deploy CKB onto target AWS EC2 instances.
function ansible_build_docker_image() {
    ansible_config

    cd $ANSIBLE_DIRECTORY
    ansible-playbook playbook.yml \
        -e 'hostname=instances' \
        -t build
}






function main() {
    case $1 in
        "run")
            job_setup
            terraform_apply
            ansible_build_docker_image
            ;;
        "setup")
            job_setup
            ;;
        "terraform")
            terraform_apply
            ;;
        "ansible")
            ansible_build_docker_image
            ;;
        "clean")
            terraform_destroy
            job_clean
            ;;
        esac
}

main $*
