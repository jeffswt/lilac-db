# Keqing must be run from the shell script. Running it directly through GNU
# Make is not recommended.
# Current working directory must be at repository root.

# External arguments
ARG_COMMIT_MSG  = $(_ARG_COMMIT_MSG)

# Applications
BIN_BASH        = bash
BIN_CURL        = curl
BIN_GIT         = git
BIN_MAKE        = make
BIN_MKDIR       = mkdir
BIN_PERL        = perl
BIN_PYTHON3     = python3
BIN_RUSTC       = rustc
BIN_TAR         = tar
BIN_TOUCH       = touch

# Resource definitions
CWD             = $(shell pwd)
KEQING          = ./keqing
ARTIFACTS       = $(KEQING)/.artifacts
RULES           = $(KEQING)/rules

# Action references
ACTION_DEPENDS   = $(BIN_MAKE) --makefile="$(RULES)/keqing.mk"

# Index of target rules
include $(RULES)/git/archive.mk
include $(RULES)/git/branch.mk
include $(RULES)/git/checkout.mk
include $(RULES)/git/commit.mk
include $(RULES)/git/workspace.mk

include $(RULES)/install/environ.mk

include $(RULES)/style/branch-commit-msgs.mk
include $(RULES)/style/branch-name.mk
