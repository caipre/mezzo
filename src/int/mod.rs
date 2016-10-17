mod idt;
mod handlers;

lazy_static! {
        static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(0, handlers::divide_by_zero);
        idt
    };
}

pub fn init() {
    IDT.load();
}
