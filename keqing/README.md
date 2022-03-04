# Keqing

Keqing is a local CI (continuous integration) tool designed for LilacDB to work
elegantly, and locally. You can trust Yuheng of the Liyue Qixing for
safeguarding against ugly code, untested code, or bad code.

> Issues affecting humans' fate should be handled by humans, and humans are
  more capable of handling them with finesse.

![Keqing, Yuheng of the Liyue Qixing](./docs/keqing_character_art.jpg)

## Introduction

Keqing is capable of performing tradition CI stuff like running release builds,
batch unit tests, integration tests or even to code style validations locally,
without the aid of an external CI server or CI service.

By caching artifacts under `~/keqing/.artifacts` (whereas `~` stands for the
repository root), Keqing can keep track of all historical workspaces, build
outputs, test results, etc., in a single directory. These 'temporary' files may
be cleaned at any time, and are 100% recoverable by re-executing the same
commands (or effectively).

These artifacts are managed with builtin Bash scripts or GNU Make rules to
minimize the need to perform rebuilds. In fact, when all actions are performed
successfully, little to no redundant work are inflicted.

## Example

A typical workflow might look like:

```bash
alias keqing=./keqing/keqing.sh
keqing branch new-feature

# write some code
keqing commit "feat(*): some new feature"
# write some more code
keqing commit "fix(*): undefined error"
# write even more code...

# ready to merge
keqing run pr gate
keqing merge with message "merge(*): add a certain new feature"
git push
git branch -d new-feature
```

For detailed usage, see *Usage* section.

## Usage

Keqing should be run under Linux (WSL is also fine). It currently works with
Bash, while Ksh and Zsh haven't been tested yet.

Provide arguments to `~/keqing/keqing.sh` to execute commands:

### Miscellany

- `help`: Show current help
- `setup environment`: Setup toolchain requirements (e.g. packages).
- `clean`: Clean all temporary build files (this will not clean up the code).

### Version Control

- `checkout [branch]`: Switch current repository HEAD to _branch_.
- `create branch [branch]`: Create a new branch _branch_ based off _master_.
- `commit [message]`: Commit with conventional commits style validation on
  _message_.
- `check merge conflicts`: Check if the current branch has merge conflicts
  against _master_. If conflicts occur, these conflicts are preserved in the
  workspace to be manually resolved.

### Build and Test

- `fix styles`: Run auto style fixes on current workspace. It is suggested to
  ensure that all changes had been staged.
- `build release`: Perform full build on current workspace.
- `build release on commit [commit]`: Generate build artifacts for a historical
  commit _commit_.
- `build release on branch [branch]`: Generate build artifacts for the latest
  commit (HEAD) on branch _branch_.
- `run tests`: Run tests (unit tests and integration tests) on current
  workspace.
- `run tests on commit [commit]`: Get test reports for a historical commit
  _commit_.
- `run tests on branch [branch]`: Get test reports for the latest commit (HEAD)
  on branch _branch_.

### Style Checks

- `check styles`: Run style checks (lint) on current workspace.
- `check styles on commit [commit]`: Get linters' code style reports for a
  historical commit _commit_.
- `check styles on branch [branch]`: Get linters' code style reports for the
  latest commit (HEAD) on branch _branch_.
- `check commit messages`: Validate if all new commits on the current branch
  adhere to the conventional commits standard.
- `check commit messages on branch [branch]`: Validate if all new commits on
  branch _branch_ adhere to the conventional commits standard.

### Code Check-in

- `run pr gate`: Ensure that the current branch is 100% ready to check-in. If
  merge conflicts occur, these conflicts are preserved in the workspace to be
  manually resolved.
- `merge with message [message]`: Check-in all code on current branch to
  _master_. This will not delete the current branch. If merge conflicts occur,
  these conflicts are preserved in the workspace to be manually resolved,
  before proceeding with the merge. It is advised that merge conflicts be
  resolved on the current branch instead of _master_.

## Trivia

- When a new commit scope is introduced, it should be manually added to the
  index in file `~/keqing/rules/style/commit-msg.py`, located in the enum class
  `CommitScope`.
- Using an alias for `./keqing/keqing.sh` is better than typing the whole
  filename in every time.
