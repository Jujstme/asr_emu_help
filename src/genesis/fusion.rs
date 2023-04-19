use asr::Address;
use asr::signature::Signature;

use crate::genesis::Endianess;

pub fn fusion(game: &mut crate::genesis::ProcessInfo) -> Option<Address> {
    const NAME: &str = "Fusion.exe";
    const SIG: Signature<4> = Signature::new("75 2F 6A 01");
    let proc = &game.emulator_process;

    let main_module_base = proc.get_module_address(NAME).ok()?;
    let main_module_size = proc.get_module_size(NAME).ok()?;
    let ptr = SIG.scan_process_range(proc, main_module_base, main_module_size)?.0 + 1;

    let addr = ptr + proc.read::<u8>(Address(ptr)).ok()? as u64 + 3;
    let addr = proc.read::<u32>(Address(addr)).ok()?;
    
    game.endianess = Endianess::BigEndian;

    Some(Address(addr as u64))
}

pub fn keep_alive() -> bool {
    true
}