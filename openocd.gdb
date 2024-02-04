target remote :3333
layout src
layout asm
layout split
load
monitor arm semihosting enable
break main
break 97
break loop
