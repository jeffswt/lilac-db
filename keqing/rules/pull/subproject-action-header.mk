# This makefile is used to initiate an action within a subproject, with
# prepended arguments and headers.

# RESERVED MAKEFILE ARGUMENTS
# root of subproject
ARG_SUBPROJ_ROOT       = $(_ARG_SUBPROJ_ROOT)
# target directory to store artifacts
ARG_ARTIFACT_ROOT      = $(_ARG_ARTIFACT_ROOT)
# included subproject makefile path
ARG_SUBPROJ_MAKEFILE   = $(_ARG_SUBPROJ_MAKEFILE)
# arguments file path
ARG_ARGUMENTS_MAKEFILE = $(_ARG_ARGUMENTS_MAKEFILE)

include $(ARG_ARGUMENTS_MAKEFILE)
include $(ARG_SUBPROJ_MAKEFILE)
