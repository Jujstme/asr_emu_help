use asr::Address;
use asr::signature::Signature;
use super::Endianness;

pub fn segaclassics(game: &mut super::ProcessInfo) -> Option<Address> {
    const SIG_GAMEROOM: Signature<20> = Signature::new("C7 05 ???????? ???????? A3 ???????? A3 ????????");
    const SIG_SEGACLASSICS: Signature<12> = Signature::new("89 2D ???????? 89 0D ????????");

    let proc = &game.emulator_process;

    let (Some(main_module_address), Some(main_module_size)) = super::PROCESS_NAMES.iter()
        .filter(|p| p.1 == super::Emulator::SegaClassics)
        .map(|m| (proc.get_module_address(m.0).ok(), proc.get_module_size(m.0).ok()))
        .find(|m| m.0.is_some() && m.1.is_some())? else { return None };

    let mut ptr = if let Some(x) = SIG_GAMEROOM.scan_process_range(proc, main_module_address, main_module_size) {
        x.0 + 2
    } else {
        SIG_SEGACLASSICS.scan_process_range(proc, main_module_address, main_module_size)?.0 + 8
    };

    ptr = proc.read::<u32>(Address(ptr)).ok()? as u64;
    ptr = proc.read::<u32>(Address(ptr)).ok()? as u64;

    game.endianess = Endianness::LittleEndian;

    Some(Address(ptr))
}

pub fn keep_alive() -> bool {
    true
}