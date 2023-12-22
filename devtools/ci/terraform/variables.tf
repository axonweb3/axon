variable "access_key" {
  type        = string
  description = "AWS access key"
}

variable "secret_key" {
  type        = string
  description = "AWS secret key"
}

variable "region" {
  type        = string
  default     = "ap-southeast-1"
  description = "AWS region"
}

variable "public_key_path" {
  type        = string
  description = "local path to ssh public key"
}

variable "private_key_path" {
  type        = string
  description = "local path to ssh private key"
}

variable "prefix" {
  type        = string
  description = "prefix attach to resource names"
}

variable "instance_type" {
  type    = string
  default = "c6g.xlarge"
}

variable "username" {
  type    = string
  default = "ec2-user"
}

variable "private_ip_prefix" {
  type    = string
  default = "10.0.1"
}
