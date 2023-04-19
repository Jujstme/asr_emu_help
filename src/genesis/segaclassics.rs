use asr::Address;
use asr::signature::Signature;

use crate::genesis::Endianess;

pub fn segaclassics(game: &mut crate::genesis::ProcessInfo) -> Option<Address> {
    const NAME: [&str; 2] = ["SEGAGameRoom.exe", "SEGAGenesisClassics.exe"];
    const SIG_GAMEROOM: Signature<20> = Signature::new("C7 05 ???????? ???????? A3 ???????? A3 ????????");
    const SIG_SEGACLASSICS: Signature<12> = Signature::new("89 2D ???????? 89 0D ????????");
    let proc = &game.emulator_process;

    let main_module_base: Address;
    let main_module_size: u64;
    let mut ptr: u64;
    
    if let Ok(x) = proc.get_module_address(NAME[0]) {
        main_module_base = x;
        main_module_size = proc.get_module_size(NAME[0]).ok()?;
        ptr = SIG_GAMEROOM.scan_process_range(proc, main_module_base, main_module_size)?.0 + 2;
    } else {
        main_module_base = proc.get_module_address(NAME[1]).ok()?;
        main_module_size = proc.get_module_size(NAME[1]).ok()?;
        ptr = SIG_SEGACLASSICS.scan_process_range(proc, main_module_base, main_module_size)?.0 + 8;
    };

    ptr = proc.read::<u32>(Address(ptr)).ok()? as u64;
    ptr = proc.read::<u32>(Address(ptr)).ok()? as u64;

    game.endianess = Endianess::LittleEndian;

    Some(Address(ptr))
}

pub fn keep_alive() -> bool {
    true
}