use asr::Address;
use asr::signature::Signature;

pub fn epsxe(game: &crate::ps1::ProcessInfo) -> Option<Address> {
    const NAME: &str = "ePSXe.exe";
    const SIG: Signature<5> = Signature::new("C1 E1 10 8D 89");
    let proc = &game.emulator_process;

    let main_module_base = proc.get_module_address(NAME).ok()?;
    let main_module_size = proc.get_module_size(NAME).ok()?;

    let ptr = SIG.scan_process_range(proc, main_module_base, main_module_size)?.0 + 5;
    let ptr = proc.read::<u32>(Address(ptr)).ok()? as u64;
    Some(Address(ptr))
}

pub fn keep_alive() -> bool {
    true
}