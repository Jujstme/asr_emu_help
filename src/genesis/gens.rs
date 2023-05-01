use asr::{Address, signature::Signature, primitives::dynamic_endian::Endian};

pub fn gens(game: &mut super::ProcessInfo) -> Option<Address> {
    const SIG: Signature<10> = Signature::new("72 ?? 81 ?? FF FF 00 00 66 8B");
    let proc = &game.emulator_process;

    let (Some(main_module_address), Some(main_module_size)) = super::PROCESS_NAMES.iter()
        .filter(|p| p.1 == super::Emulator::Gens)
        .map(|m| (proc.get_module_address(m.0).ok(), proc.get_module_size(m.0).ok()))
        .find(|m| m.0.is_some() && m.1.is_some())? else { return None };

    let ptr = SIG.scan_process_range(proc, main_module_address, main_module_size)?.0 + 11;

    game.endianness = if proc.read::<u8>(Address(ptr + 4)).ok()? == 0x86 {
        Endian::Big
    } else {
        Endian::Little
    };

    let wram = proc.read::<u32>(Address(ptr)).ok()? as u64;

    Some(Address(wram))
}

pub fn keep_alive() -> bool {
    true
}