MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 896K
  STORAGE : ORIGIN = 0x080E0000, LENGTH = 128K
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
  CCMRAM : ORIGIN = 0x10000000, LENGTH = 64K
}

/* Keep .data/.bss in SRAM and move the runtime stack into CCM RAM to avoid
   collisions with the dungeon viewport buffer. */
_stack_start = ORIGIN(CCMRAM) + LENGTH(CCMRAM);
_stack_end = ORIGIN(CCMRAM);
