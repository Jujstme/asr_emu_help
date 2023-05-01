use asr::{Address, signature::Signature, sync::Mutex, primitives::dynamic_endian::Endian};

static STATICDATA: Mutex<StaticData> = Mutex::new(StaticData {
    addr: Address(0),
});

struct StaticData {
    addr: Address,
}

pub fn fusion(game: &mut super::ProcessInfo) -> Option<Address> {
    const SIG: Signature<4> = Signature::new("75 2F 6A 01");
    let proc = &game.emulator_process;

    let (Some(main_module_address), Some(main_module_size)) = super::PROCESS_NAMES.iter()
        .filter(|p| p.1 == super::Emulator::Fusion)
        .map(|m| (proc.get_module_address(m.0).ok(), proc.get_module_size(m.0).ok()))
        .find(|m| m.0.is_some() && m.1.is_some())? else { return None };

    let ptr = SIG.scan_process_range(proc, main_module_address, main_module_size)?.0 + 1;

    let addr = ptr + proc.read::<u8>(Address(ptr)).ok()? as u64 + 3;
    let addr = Address(proc.read::<u32>(Address(addr)).ok()? as u64);

    let mut static_data = STATICDATA.lock();
    static_data.addr = addr;

    let addr = proc.read::<u32>(addr).ok()?;
    
    game.endianness = Endian::Big;

    Some(Address(addr as u64))
}

pub fn keep_alive(game: &mut super::ProcessInfo) -> bool {
    let static_data = STATICDATA.lock();

    if let Ok(addr) = game.emulator_process.read::<u32>(static_data.addr) {
        game.wram_base = Some(Address(addr as u64));
        true
    } else {
        false
    }
}