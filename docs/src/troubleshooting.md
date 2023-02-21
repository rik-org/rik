# Troubleshooting

**`cargo build` fails because cannot build `openssl-sys`**

This is due to missing packages in your system, install `libssl-dev` to fix this.

- Ubuntu: `sudo apt update && sudo apt install libssl-dev protobuf-compiler`

**protoc failed: Explicit 'optional' labels are disallowed in the Proto3 
Syntax**

This is due to the version of `protoc` you are using, you need to use version
3.14 or later.