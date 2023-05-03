CS.D stands for the "default operation size" field of the segment descriptor associated with the current Code Segment. This controls the default size of addresses and operands, and can be set to default to 16 or 32 bit operand-size.

In 64-bit aka long mode (CS.L=1), the only valid setting for CS.D = 32-bit, so a REX prefix with the W bit cleared leaves the default operand size at 32-bit. (An operand-size prefix can override the operand-size down to 16).

The default address size in long mode is 64-bit (an address-size prefix on an instruction overrides it to 32).

Segment descriptors are described in detail in Volume 3A - System Programming Guide, Part 1, chapter 3.4.5 Segment Descriptors.

The effects of the D field are also discussed in Volume 1 - Basic Architecture, chapter 3.6 Operand-size and address-size attributes.

