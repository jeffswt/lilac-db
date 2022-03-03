# Displays a prettified message box indicating which task is running right now.

# external arguments
bin_echo=${1}

stage=${2}   # no. of task, starting from 1
title=${3}   # title of stage
message=${4} # description

# write stuff
"$bin_echo" "                                                         "
"$bin_echo" "+--------------------------------------------------------"
"$bin_echo" "|  PR GATE [$stage] - $title"
"$bin_echo" "|                                                        "
"$bin_echo" "|     $message"
"$bin_echo" "+--------------------------------------------------------"
"$bin_echo" "                                                         "
