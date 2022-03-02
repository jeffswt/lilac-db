# Keqing must be run from the shell script. Running it directly through GNU
# Make is not recommended.
# Current working directory must be at repository root.

# External arguments
ARG_COMMIT_MSG  = $(_ARG_COMMIT_MSG)

# Resource definitions
KEQING          = ./keqing
ARTIFACTS       = $(KEQING)/.artifacts
RULES           = $(KEQING)/rules

# binary declarations
include $(RULES)/arguments.mk

# depend on this to be phony targets
include $(RULES)/always-execute.mk

# Action references
ACTION_DEPENDS  = "$(BIN_MAKE)" --makefile="$(RULES)/keqing.mk" --silent
ACTION_SUBPROJ  = $(__PULL__SUBPROJECT_ACTION)

# Index of target rules
include $(RULES)/fix/style.mk

include $(RULES)/git/archive.mk
include $(RULES)/git/branch.mk
include $(RULES)/git/checkout.mk
include $(RULES)/git/commit.mk
include $(RULES)/git/no-merge-conflicts.mk
include $(RULES)/git/staged-all.mk
include $(RULES)/git/workspace.mk

include $(RULES)/install/environ.mk

include $(RULES)/pull/subproject-action.mk

include $(RULES)/style/branch-commit-msgs.mk
include $(RULES)/style/branch-name.mk
include $(RULES)/style/rustfmt.mk
