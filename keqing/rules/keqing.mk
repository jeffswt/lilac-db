# Keqing must be run from the shell script. Running it directly through GNU
# Make is not recommended.
# Current working directory must be at repository root.

# External arguments
ARG_BRANCH      = $(_ARG_BRANCH)
ARG_COMMIT_HASH = $(_ARG_COMMIT_HASH)
ARG_COMMIT_MSG  = $(_ARG_COMMIT_MSG)

# All variable definitions
KEQING          = ./keqing
ARTIFACTS       = $(KEQING)/.artifacts
RULES           = $(KEQING)/rules

# Applications
BIN_BASH        = bash
BIN_GIT         = git
BIN_RUSTC       = rustc

# Index of target rules
include $(RULES)/git/archive.mk
include $(RULES)/git/checkout.mk
include $(RULES)/git/commit.mk
include $(RULES)/git/workspace.mk
