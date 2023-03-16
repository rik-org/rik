# Contributing guide

## Setup your developer environment

### Dependencies 

- `umoci`
- `skopeo`
- `runc`
- `protobuf-compiler`

For ubuntu :

```bash
sudo apt update
sudo apt install -y umoci skopeo runc protobuf-compiler
````

For fedora : 

```bash
sudo dnf update
sudo dnf install -y umoci skopeo runc
# To install protoc
curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v21.12/protoc-21.12-linux-x86_64.zip
unzip protoc-3.21.12-linux-x86_64 -d $HOME/.local
export PATH="$PATH:$HOME/.local/bin"
```

### Launch a rik cluster

#### Configuration

Create one file to specify the rik cluster configuration, for example `/tmp/rikconfig.yml` :
```yml
cluster:
  name: rik-demo
  server: http://127.0.0.1:5000
```

Export the `RIKCONFIG` environment variable to point to this file : 
```bash
export RIKCONFIG=/tmp/rikconfig.yml
```

You will need an example workload to test the cluster, here is one created in `/tmp/workload.json` :
```json
{
  "api_version": "v0",
  "kind": "pods",
  "name": "devopsdday-alpine",
  "spec": {
    "containers": [
      {
        "name": "alpine",
        "image": "alpine:latest"
      }
    ]
  }
}
```

#### Build and run the rik cluster

```bash
# Run & build scheduler
cd scheduler
cargo build --release
./release/scheduler

# Run & build controller
cd controller
cargo build --release
sudo ./release/controller

# Run & build riklet
cd riklet 
cargo build --release
sudo ./release/riklet

# Run & build riktl
cd riklet 
cargo build --release

# Create file rikconfig
nano /tmp/rikconfig.yml
export RIKCONFIG=/tmp/rikconfig.yml

# Create and instantiate workload
nano /tmp/workload.json
./rikctl create workload --file /tmp/workload.json
./rikctl create instance --workload-id [WORKLOAD-ID]

# Verify the container creation
sudo runc list
```

## Troubleshooting

**`cargo build` fails because cannot build `openssl-sys`**

This is due to missing packages in your system, install `libssl-dev` to fix this.

- Ubuntu: `sudo apt update && sudo apt install libssl-dev protobuf-compiler`

## Workflow

RIK is using a workflow to be efficient and preserve the quality of the application and code.

### Submit an issue

We want to follow a simple and easy way to submit issues or pull requests. 
If you want to report a bug or submit a feature request, please use the templates provided.

_Using our templates for issues and bug report, labels should be automatically provided._

### Open a Pull Request

Once you chose an issue that you want to work on, consider using templates provided and fill it.
When a precise description has been filled in you can start to open your draft Pull Request.

### Review flow

**Your pull request have to be rebased before asking for merge**

When your Pull Request is ready to be reviewed, you can declare it as ready. The Pull Request will be checked in a first step by a maintainer of the core team using the following criterias:

- Rebased and don't have merge conflict
- Code explanations and documentation provided
- Unitary tests or procedure to test the feature provided
- Pull request have a description of the feature objectives provided

If the maintainer validates the relevance and form of the Pull Request, he will approve this Pull Request or request some changes before it should be merged.

_NOTE: A pull request need 2 approvals to be merged._

## Commits conventions

Our commit convention follow the [Conventional Commits 1.0.0-beta.4](https://www.conventionalcommits.org/en/v1.0.0-beta.4/)

To be more explicit, we will describe below the commit conventions message.

### Commit Message Format

Each commit message consists of a header, a body and a footer. The header has a special format that includes a type, a scope and a subject:

```
<type>(<scope>): <subject>
<BLANK LINE>

<body>
<BLANK LINE>
<footer>
```

The header is mandatory and the scope of the header is optional.

Any line of the commit message cannot be longer 100 characters! This allows the message to be easier to read on GitHub as well as in various git tools.

The footer should contain a [closing reference to an issue](https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue) if any.

Samples: (even more samples)

```
docs(changelog): update changelog to beta.5
```

```
fix(release): need to depend on latest tonic and tower
```

The version in our package.json gets copied to the one we publish, and users need the latest of these.

### Revert

If the commit reverts a previous commit, it should begin with `revert:` , followed by the header of the reverted commit. In the body it should say: `This reverts commit <hash>.`, where the hash is the SHA of the commit being reverted.

### Type

Must be one of the following:

- **build**: Changes that affect the build system or external dependencies (example scopes: cargo, make...)
- **chore**: Some housekeeping activity
- **ci**: Changes to our CI configuration files and scripts
- **docs**: Documentation only changes
- **feat**: A new feature
- **fix**: A bug fix
- **perf**: A code change that improves performance
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **revert**: A commit revert
- **test**: Adding missing tests or correcting existing tests

### Scope

The scope should be the name of the related RIK component if applicable.

The following is the list of supported scopes:

- rikctl
- controller
- scheduler
- riklet

There is currently an exception to the "use component name" rule:

- **none/empty string**: useful for changes not directly related to a specific component (e.g. chore: fix ci)

### Subject

The subject contains a succinct description of the change:

- use the imperative, present tense: "change" not "changed" nor "changes"
- don't capitalize the first letter
- no dot (.) at the end

### Body

Just as in the subject, use the imperative, present tense: "change" not "changed" nor "changes". The body should include the motivation for the change and contrast this with previous behavior.

### Footer

The footer should contain any information about Breaking Changes and is also the place to [reference GitHub issues that this commit Closes](https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue).

**Breaking Changes** should start with the word BREAKING CHANGE: with a space or two newlines. The rest of the commit message is then used for this.

### Sign commit message

Please consider signing the commit message at least with `Signed-Off-By`. This is a way to certify that you wrote the code or otherwise have the right to pass it on as an open-source patch. The process is simple: if you can certify the [Developer's Certificate of Origin](https://developercertificate.org/) (DCO), then just add a line to every git commit message:
