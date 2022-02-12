variable "gcp_project" {
  type        = string
  description = "The project to use on Google Cloud Platform."
}

variable "gcp_region" {
  type        = string
  description = "The Google Cloud Platform region where to deploy resources"
}

variable "workers_count" {
  type        = number
  description = "The number of workers to provision."
  default     = 1
}

variable "cluster_name" {
  type        = string
  description = "The name of the cluster."
  default     = "rik"
}