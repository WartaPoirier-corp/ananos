# Context switches

## How to enter ring 3?

The idea is to trick the processor into thinking it is handling an interrupt, and that
it needs to go back to ring 3 (while it never ran ring 3 code, just ring 1), with the help
of the `iret` instruction (Interrupt RETurn, to leave kernel after an interrupt has been
handled).

Normally, when an interrupt comes in, the processor pushes a few things on the stack:

- The stack segment and pointer of the interrupted process (`SS:ESP`, Stack Segment : E (= 32bits) Stack Pointer)
- `EFLAGS`, a (32bits wide) register containing the state of the processor[^eflags]
- The code segment and instruction pointer of the interrupted code, to get back to it once the
  interrupt is handled (`CS:EIP`, Code Segement : E Instruction Pointer)
- An error code, for the interrupts that have one

The `iret` instructions uses this information to get back to the code that was being executed before the interrupt. So the idea here is to push "fake" information (pointing to the ring 3 code we want to run) and to call `iret`, so that the processor runs it in ring 3.

In assembly it looks like that (stolen on the OS dev wiki, added a lot of comments (which may be wrong, it is only what I understood so far)):

```asm
GLOBAL _jump_usermode ; Export this function to be able to call it from C or Rust
EXTERN _test_user_function ; The test function to run in ring 3

_jump_usermode:
     ; First, make all data registers point to the user data segment selector
     ; This selector has been fixed to 0x23 in this example.
     ; It should be set up on the GDT before that, and actual program data
     ; should be stored at this address
     mov ax,0x23
     mov ds,ax
     mov es,ax 
     mov fs,ax 
     mov gs,ax
 
     ; Now we do the "hack" to trap iret
     mov eax,esp ; First, save the current Stack Pointer
     push 0x23 ; Push the Stack Segment (0x23, the same as the data segments in this example (because there is no program data in this case, I guess?))
     push eax ; Push the Stack Pointer that was saved (the ring 3 and ring 0 code thus share the same stack here)
     ; So at this point we have SS:ESP on the stack
     pushf ; Now push the FLAGS register on the stack (TODO: isn't it supposed to be EFLAGS?? like pushfd)
     push 0x1B ; Push the user Code Segment (here too it should be set up in the GDT before, normally)
     push _test_user_function ; Push the Instruction Pointer of the function to call
     ; We have SS, SP, EFLAGS, CS and EIP on the stack, we can call iret!
     iret
;end
```

Little problem with the assembly above: interrupts are enabled and could stop us in the middle of a context switch. `cli` (the instruction that normally disable interrupts) can't be used, as there is now way to restore them once `iret` has been called (`sti`, the instruction to re-enable interrupts is not allowed in ring 3).

Fortunately there is a trick that saves us here too: `sti` just sets the `IF` flag in the `FLAGS` register, so we can just change this flag in the "fake" `EFLAGS` we are pushing on the stack, so that as we enter ring 3, these flags gets restored, with interrupts enabled (and we will need to add a `cli` at the beginning of the procedure too). The code to do that is:

```asm
pushf ; same line as in the code above

; now the new code:
pop %eax ; Get EFLAGS back into EAX.
or %eax, $0x200 ; Set the IF flag.
push %eax ; Push the new EFLAGS value back onto the stack.

; push CS, EIP, and do iret…
```

And now the real question: how to do that in Rust?

Edit: to be short, with `asm!`.

## Actual multi-processing

This allows to run some code in ring 3, but in the real world, this won't be enough.
Interrupts (real ones this time) may suddently appear, stopping our ring 3 code.
Real world OSes need to run more than one user process, and thus to do task switching
regularly.

Thus, the kernel needs some data about each process:

- the values of the general purpose registers, to save/restore them on context switches
- the SS:SP, and data register values
- the CS:IP registers
- the "page register" (CR3)
- a dedicated kernel stack to handle interrupts : if all the processes had the same kernel stack, you could end up in "funny" situations if an interrupt is fired while another one was being handled (and probably some other weird issues).

## Problems that were encountered during implementation of the POC

I'm documenting that, it may help future OS-hobbyist (finding resources is sometimes hard).

First of all, set up handler for page faults and general protection faults, as you will probably
get some of them at sooner or later. It helps a lot to debug your kernel IMO. And give them their
own stacks with the TSS.

I first tried the approach described above, thinking that my user code would be able to
share the same stack and code as the kernel. But I was getting page faults because
the bootloader maps a kernel stack and code that don't have the `USER_ACCESSIBLE` bit.

I decided to map two new pages with the code and the stack for the process, with all the
correct bits. I wrote some fake program and copied it to the code page (`0x90` is the x86
opcode for `nop`, which can help when writing fake programs, it just makes the IP go up).

Then it worked, but the code never stopped executing, causing a fault again, because `0x0` is not
a valid opcode. So I decided to add a "syscall" to stop a program, using `int 0x0`, as Linux does
for actual syscalls.

This required me to set up a new interrupt handler at index `0x80` (not `80`, don't make the same mistakes as me, if you are getting a General Protection Fault with a code of `0x802`, it means it
couldn't find the `0x80` handler, check that you didn't mistyped anything). This handler also need
its own stack in the TSS. And at the end of the handler I just `loop {}`ed to block and avoid continuing program execution (later, the programm will actually get terminated).

## Sources

- <https://code-examples.net/en/q/692b85>
- <https://wiki.osdev.org/Getting_to_Ring_3>
- <http://www.jamesmolloy.co.uk/tutorial_html/10.-User%20Mode.html>

[^eflags]: https://en.wikipedia.org/wiki/FLAGS_register
