mod idt;

use vga::kerror;

macro_rules! handler {
    ($name:ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("mov rdi, rsp
                    sub rsp, 8
                    call $0"
                    :: "i"($name as extern "C" fn(*const ExceptionStackFrame) -> !)
                    : "rdi" : "intel");
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
                asm!("
                    pop rsi
                    mov rdi, rsp
                    sub rsp, 8
                    call $0"
                    :: "i"($name as extern "C" fn(*const ExceptionStackFrame, u64) -> !)
                    : "rdi", "rsi" : "intel");
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

extern "C" fn divide_by_zero(stack_frame: *const ExceptionStackFrame) -> ! {
    unsafe {
        kerror(format_args!("division by zero\n{:#?}", *stack_frame));
    };
    loop {}
}

extern "C" fn invalid_opcode(stack_frame: *const ExceptionStackFrame) -> ! {
    unsafe {
        kerror(format_args!("invalid opcode at {:#x}\n{:#?}",
                            (*stack_frame).ip, *stack_frame));
    };
    loop {}
}

extern "C" fn page_fault(stack_frame: *const ExceptionStackFrame, error_code: u64) -> ! {
    use x86::shared::control_regs;
    unsafe {
        kerror(format_args!("page fault accessing {:#x} ({:?})\n{:#?}",
                            control_regs::cr2(),
                            PageFaultErrorCode::from_bits(error_code).unwrap(),
                            *stack_frame));
    };
    loop {}
}
