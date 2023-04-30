use asr::{Process, Address, signature::Signature};

pub fn check_for_64_bit(proc: &Process, main_module_base: Address) -> bool {
    const SIG_64: Signature<5> = Signature::new("50 45 00 00 64");
    SIG_64.scan_process_range(proc, main_module_base, 0x1000).is_some()
}