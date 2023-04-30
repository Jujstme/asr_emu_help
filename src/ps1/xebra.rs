use asr::Address;
use asr::signature::Signature;

pub fn xebra(game: &super::ProcessInfo) -> Option<Address> {
    const NAME: &str = "XEBRA.EXE";
    const SIG: Signature<15> = Signature::new("E8 ?? ?? ?? ?? E9 ?? ?? ?? ?? 89 C8 C1 F8 10");
    let proc = &game.emulator_process;

    let main_module_base = proc.get_module_address(NAME).ok()?;
    let main_module_size = proc.get_module_size(NAME).ok()?;

    let ptr = SIG.scan_process_range(proc, main_module_base, main_module_size)?.0 as i32 + 1;
    let addr = ptr + 0x4 + proc.read::<i32>(Address(ptr as u64)).ok()?;
    let addr = proc.read::<i32>(Address(addr as u64 + 0x16A)).ok()?;
    let addr = proc.read::<i32>(Address(addr as u64)).ok()?;
    Some(Address(addr as u64))
}

pub fn keep_alive() -> bool {
    true
}