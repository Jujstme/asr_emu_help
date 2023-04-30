use asr::Address;
use asr::signature::Signature;

pub fn epsxe(game: &super::ProcessInfo) -> Option<Address> {
    const SIG: Signature<5> = Signature::new("C1 E1 10 8D 89");
    let proc = &game.emulator_process;  
    
    let (Some(main_module_address), Some(main_module_size)) = super::PROCESS_NAMES.iter()
        .filter(|p| p.1 == super::Emulator::Epsxe)
        .map(|m| (proc.get_module_address(m.0).ok(), proc.get_module_size(m.0).ok()))
        .find(|m| m.0.is_some() && m.1.is_some())? else { return None };

    
    let ptr = SIG.scan_process_range(proc, main_module_address, main_module_size)?.0 + 5;
    let ptr = proc.read::<u32>(Address(ptr)).ok()? as u64;
    Some(Address(ptr))
}

pub fn keep_alive() -> bool {
    true
}