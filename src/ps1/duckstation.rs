use asr::{Address, signature::Signature};

static STATICDATA: spinning_top::Spinlock<StaticData> = spinning_top::const_spinlock(StaticData {
    addr: Address(0),
});

struct StaticData {
    addr: Address,
}

pub fn duckstation(game: &super::ProcessInfo) -> Option<Address> {
    const SIG: Signature<8> = Signature::new("48 89 0D ?? ?? ?? ?? B8");
    let proc = &game.emulator_process;

    let (Some(main_module_address), Some(main_module_size)) = super::PROCESS_NAMES.iter()
        .filter(|p| p.1 == super::Emulator::Duckstation)
        .map(|m| (proc.get_module_address(m.0).ok(), proc.get_module_size(m.0).ok()))
        .find(|m| m.0.is_some() && m.1.is_some())? else { return None };

    let addr = SIG.scan_process_range(proc, main_module_address, main_module_size)?.0 as i64 + 3;
    let ptr = addr + 0x4 + proc.read::<i32>(Address(addr as u64)).ok()? as i64;

    let mut static_data = STATICDATA.lock();
    static_data.addr = Address(ptr as u64);
    let wram = proc.read::<u64>(static_data.addr).ok()?;

    Some(Address(wram))
}

pub fn keep_alive(game: &mut super::ProcessInfo) -> bool {
    let static_data = STATICDATA.lock();

    if let Ok(addr) = game.emulator_process.read::<u64>(static_data.addr) {
        game.wram_base = Some(Address(addr));
        true
    } else {
        false
    }
}