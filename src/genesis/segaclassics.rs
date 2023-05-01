use asr::{Address, signature::Signature, primitives::dynamic_endian::Endian, sync::Mutex};

static STATICDATA: Mutex<StaticData> = Mutex::new(StaticData {
    addr: Address(0),
});

struct StaticData {
    addr: Address,
}

pub fn segaclassics(game: &mut super::ProcessInfo) -> Option<Address> {
    const SIG_GAMEROOM: Signature<16> = Signature::new("C7 05 ???????? ???????? A3 ???????? A3");
    const SIG_SEGACLASSICS: Signature<8> = Signature::new("89 2D ???????? 89 0D");
    const GENESISWRAPPERDLL: &str = "GenesisEmuWrapper.dll";

    let proc = &game.emulator_process;

    let mut ptr: u64 = 0;


    if let (Ok(module), Ok(size)) = (proc.get_module_address(GENESISWRAPPERDLL), proc.get_module_size(GENESISWRAPPERDLL)) {
        ptr = SIG_GAMEROOM.scan_process_range(proc, module, size)?.0 + 2;
    } else {
        let (Some(main_module_address), Some(main_module_size)) = super::PROCESS_NAMES.iter()
            .filter(|p| p.1 == super::Emulator::SegaClassics)
            .map(|m| (proc.get_module_address(m.0).ok(), proc.get_module_size(m.0).ok()))
            .find(|m| m.0.is_some() && m.1.is_some())? else { return None };

        ptr = SIG_SEGACLASSICS.scan_process_range(proc, main_module_address, main_module_size)?.0 + 8;
    }

    ptr = proc.read::<u32>(Address(ptr)).ok()? as u64;

    let mut static_data = STATICDATA.lock();
    static_data.addr = Address(ptr);
    game.endianness = Endian::Little;

    ptr = proc.read::<u32>(static_data.addr).ok()? as u64;

    Some(Address(ptr))
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