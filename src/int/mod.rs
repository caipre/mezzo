mod idt;

use vga::kerror;

macro_rules! save_scratch_registers {
    () => {
        asm!("
             push rax
             push rcx
             push rdx
             push rsi
             push rdi
             push r8
             push r9
             push r10
             push r11
        " :::: "intel", "volatile");
    }
}

macro_rules! restore_scratch_registers {
    () => {
        asm!("
             pop r11
             pop r10
             pop r9
             pop r8
             pop rdi
             pop rsi
             pop rdx
             pop rcx
             pop rax
        " :::: "intel", "volatile");
    }
}

macro_rules! handler {
    ($name:ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                save_scratch_registers!();
                asm!("
                    mov rdi, rsp
                    add rdi, 9*8
                    call $0"
                    :: "i"($name as extern "C" fn(*const ExceptionStackFrame))
                    : "rdi" : "intel", "volatile");
                restore_scratch_registers!();

                asm!("
                     iretq"
                     :::: "intel", "volatile");

                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    }}
}

macro_rules! error_code_handler {
    ($name:ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                save_scratch_registers!();
                asm!("
                    mov rsi, [rsp + 9*8]
                    mov rdi, rsp
                    add rdi, 10*8
                    sub rsp, 8
                    call $0
                    add rsp, 8"
                    :: "i"($name as extern "C" fn(*const ExceptionStackFrame, u64))
                    : "rdi", "rsi" : "intel");
                restore_scratch_registers!();

                asm!("
                     add rsp, 8
                     iretq"
                     :::: "intel", "volatile");

                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    }}
}


lazy_static! {
        static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(0, handler!(divide_by_zero));
        idt.set_handler(3, handler!(breakpoint));
        idt.set_handler(6, handler!(invalid_opcode));
        idt.set_handler(14, error_code_handler!(page_fault));
        idt
    };
}

pub fn init() {
    IDT.load();
}

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    ip: u64,
    cs: u64,
    flags: u64,
    sp: u64,
    ss: u64,
}

bitflags! {
    flags PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1 << 0,
        const CAUSED_BY_WRITE      = 1 << 1,
        const USER_MODE            = 1 << 2,
        const MALFORMED_TABLE      = 1 << 3,
        const INSTRUCTION_FETCH    = 1 << 4,
    }
}

extern "C" fn divide_by_zero(stack_frame: *const ExceptionStackFrame) {
    unsafe {
        kerror(format_args!("division by zero\n{:#?}", *stack_frame));
    };
    loop {}
}

extern "C" fn invalid_opcode(stack_frame: *const ExceptionStackFrame)  {
    unsafe {
        kerror(format_args!("invalid opcode at {:#x}\n{:#?}",
                            (*stack_frame).ip, *stack_frame));
    };
    loop {}
}

extern "C" fn breakpoint(stack_frame: *const ExceptionStackFrame)  {
    unsafe {
        kerror(format_args!("breakpoint at {:#x}\n{:#?}",
                            (*stack_frame).ip, *stack_frame));
    };
}

extern "C" fn page_fault(stack_frame: *const ExceptionStackFrame, error_code: u64) {
    use x86::shared::control_regs;
    unsafe {
        kerror(format_args!("page fault accessing {:#x} ({:?})\n{:#?}",
                            control_regs::cr2(),
                            PageFaultErrorCode::from_bits(error_code).unwrap(),
                            *stack_frame));
    };
    loop {}
}
