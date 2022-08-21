extern kernel_main_new_stack
extern KERNEL_MAIN_STACK

global kernel_main
kernel_main:
  mov rsp, KERNEL_MAIN_STACK + 1024 * 1024
  call kernel_main_new_stack
.fin:
  hlt
  jmp .fin
