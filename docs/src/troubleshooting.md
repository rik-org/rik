# Troubleshooting

**`cargo build` fails because cannot build `openssl-sys`**

This is due to missing packages in your system, install `libssl-dev` to fix this.

- Ubuntu: `sudo apt update && sudo apt install libssl-dev protobuf-compiler`
- Fedora: `sudo dnf update && sudo dnf install -y openssl-devel protobuf-compiler protobuf-devel`

**protoc failed: Explicit 'optional' labels are disallowed in the Proto3 
Syntax**

This is due to the version of `protoc` you are using, you need to use version
3.14 or later.

**controller failed to run: panic, Permission denied**

Controller component tries to create a folder in `/var/lib/rik/data` to store
your cluster data. You can either run the controller as root or change the saved
directory by setting `DATABASE_LOCATION` to another folder location.