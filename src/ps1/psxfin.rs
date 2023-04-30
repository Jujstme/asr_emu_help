use asr::{Address, signature::Signature};

pub fn psxfin(game: &super::ProcessInfo) -> Option<Address> {
    const SIG: Signature<9> = Signature::new("8B 15 ?? ?? ?? ?? 8D 34 1A"); // v1.13
    const SIG_0: Signature<8> = Signature::new("A1 ?? ?? ?? ?? 8D 34 18"); // v1.12
    const SIG_1: Signature<9> = Signature::new("A1 ?? ?? ?? ?? 8B 7C 24 14"); // v1.5 through v1.11
    const SIG_2: Signature<8> = Signature::new("A1 ?? ?? ?? ?? 8B 6C 24"); // v1.0 through v1.4

    let proc = &game.emulator_process;

    let (Some(main_module_address), Some(main_module_size)) = super::PROCESS_NAMES.iter()
        .filter(|p| p.1 == super::Emulator::PsxFin)
        .map(|m| (proc.get_module_address(m.0).ok(), proc.get_module_size(m.0).ok()))
        .find(|m| m.0.is_some() && m.1.is_some())? else { return None };

    let mut ptr: u64;

    if let Some(sig) = SIG.scan_process_range(proc, main_module_address, main_module_size) {
        ptr = proc.read::<u32>(Address(sig.0 + 2)).ok()? as u64;
    } else if let Some(sig) = SIG_0.scan_process_range(proc, main_module_address, main_module_size) {
        ptr = proc.read::<u32>(Address(sig.0 + 1)).ok()? as u64;
    } else if let Some(sig) = SIG_1.scan_process_range(proc, main_module_address, main_module_size) {
        ptr = proc.read::<u32>(Address(sig.0 + 1)).ok()? as u64;
    } else if let Some(sig) = SIG_2.scan_process_range(proc, main_module_address, main_module_size) {
        ptr = proc.read::<u32>(Address(sig.0 + 1)).ok()? as u64;
    } else {
        return None
    }

    ptr = proc.read::<u32>(Address(ptr)).ok()? as u64;

    if ptr == 0 {
        None
    } else {
        Some(Address(ptr))
    }
}

pub fn keep_alive() -> bool {
    true
}