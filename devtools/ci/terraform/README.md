# Terraform Configuration Files Used For Build arm64 image Workflow

These Terraform configuration files are part of the ["Build arm64 image" workflow](../../../.github/workflows/build-arm64-image.yml).

## AMI

Read [`ami.tf`](./ami.tf)

## Variables

Read [`variables.tf`](./variables.tf)

## Resources

* instances nodes named `"instance-*"`
* `aws_vpc`
* `aws_key_pair`

## Outputs

* [`ansible_hosts`](./main.tf#L101)
