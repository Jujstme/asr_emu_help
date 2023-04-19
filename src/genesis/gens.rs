use asr::Address;
use asr::signature::Signature;
use crate::genesis::Endianess;

pub fn gens(game: &mut crate::genesis::ProcessInfo) -> Option<Address> {
    const NAME: &str = "gens.exe";
    const SIG: Signature<15> = Signature::new("72 ?? 81 ?? FF FF 00 00 66 8B ?? ????????");
    let proc = &game.emulator_process;

    let main_module_base = proc.get_module_address(NAME).ok()?;
    let main_module_size = proc.get_module_size(NAME).ok()?;
    let ptr = SIG.scan_process_range(proc, main_module_base, main_module_size)?.0 + 11;

    if proc.read::<u8>(Address(ptr + 4)).ok()? == 0x86 {
        game.endianess = Endianess::BigEndian
    } else {
        game.endianess = Endianess::LittleEndian
    }

    let wram = proc.read::<u32>(Address(ptr)).ok()? as u64;
    Some(Address(wram))
}

pub fn keep_alive() -> bool {
    true
}