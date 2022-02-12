# Provision a RIK cluster on GCP

This folder provides a Terraform script to provision a RIK cluster on Google Cloud Platform.

## Requirements

**You must do the following on a linux based distribution such as Ubuntu or Debian. At the moment, other platforms are not supported.**

To use these scripts, you should install the following requirements :

- [gcloud CLI](https://cloud.google.com/sdk/gcloud)
- [Terraform](https://www.terraform.io/)
- [Cargo Deb](https://crates.io/crates/cargo-deb)

Build the project by running the following command at the root of the project :

```bash
# Build the whole project
$ cargo build --release
# Build the .deb required to install our daemons on VMs
$ cargo deb -p riklet && cargo deb -p scheduler && cargo deb -p controller
```

## GCP Setup

First, you should get credentials by login to your GCP account with the gcloud CLI :

```bash
$ gcloud auth login
```

Follow the steps provided by Google to authenticate and get your credentials.

Secondly, you have to create a project on GCP if there is no one available on your account.

## Deploy the infrastructure

To deploy the infrastructure, create a `terraform.tfvars` in the folder `integration/terraform/google-cloud` with the following content:

```
gcp_project = <YOUR_PROJECT_NAME>
gcp_region  = <YOUR_REGION>

cluster_name = "demo"
workers_count = 2
```

Now you can simply run :

```bash
$ terraform apply
```

## Get the rikconfig file

To get the `rikconfig` file, simply run :

```bash
$ terraform output -raw rikconfig > rikconfig.yml
$ export RIKCONFIG=$(pwd)/rikconfig.yml
```

Now, you should be able to interact with your RIK cluster using the [riktl](../../../riktl/README.md) command.

## Clean up

To clean up the GCP infrastructure, run the following :

```bash
$ terraform destroy
```
