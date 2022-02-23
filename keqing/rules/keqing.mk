# Keqing must be run from the shell script. Running it directly through GNU
# Make is not recommended.

# External arguments
ARG_BRANCH      = $(_ARG_BRANCH)
ARG_COMMIT_HASH = $(_ARG_COMMIT_HASH)
ARG_COMMIT_MSG  = $(_ARG_COMMIT_MSG)

# All variable definitions
KEQING          = ./keqing  # WARNING: ensure CWD at git root
ARTIFACTS       = $(KEQING)/.artifacts
RULES           = $(KEQING)/rules

# Index of target rules
include $(RULES)/git/archive.mk
include $(RULES)/git/commit.mk
include $(RULES)/git/workspace.mk
