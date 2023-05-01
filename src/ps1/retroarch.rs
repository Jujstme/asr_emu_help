use crate::shared::check_for_64_bit;
use asr::{Address, signature::Signature, sync::Mutex};

static STATICDATA: Mutex<StaticData> = Mutex::new(StaticData {
    core_addr: Address(0),
});

struct StaticData {
    core_addr: Address,
}

pub fn retroarch(game: &super::ProcessInfo) -> Option<Address> {
    const SUPPORTED_CORES: [&str; 4] = [
        "mednafen_psx_hw_libretro.dll",
        "mednafen_psx_libretro.dll",
        "swanstation_libretro.dll",
        "pcsx_rearmed_libretro.dll",
    ]; 

    let proc = &game.emulator_process;
    let mut static_data = STATICDATA.lock();

    let Some(main_module_address) = super::PROCESS_NAMES.iter()
        .filter(|p| p.1 == super::Emulator::Retroarch)
        .map(|m| proc.get_module_address(m.0).ok())
        .find(|m| m.is_some())? else { return None };

    let is_64_bit = check_for_64_bit(proc, main_module_address);

    let (&core, Ok(core_address)) = SUPPORTED_CORES.iter()
        .map(|m| (m, proc.get_module_address(m)))
        .find(|m| m.1.is_ok())? else { return None };

    static_data.core_addr = core_address;

    if core == SUPPORTED_CORES[0] || core == SUPPORTED_CORES[1] {
        // Mednafen
        if is_64_bit {
            const SIG: Signature<14> = Signature::new("48 8B 05 ?? ?? ?? ?? 41 81 E4 FF FF 1F 00");
            let ptr = SIG.scan_process_range(proc, core_address, proc.get_module_size(core).ok()?)?.0 + 3;
            let ptr = ptr as i64 + 0x4 + proc.read::<i32>(Address(ptr)).ok()? as i64;

            let ptr = proc.read::<u64>(Address(ptr as u64)).ok()?;
            Some(Address(ptr))
        } else {
            const SIG: Signature<11> = Signature::new("A1 ?? ?? ?? ?? 81 E3 FF FF 1F 00");
            let ptr = SIG.scan_process_range(proc, core_address, proc.get_module_size(core).ok()?)?.0 + 1;
            let ptr = proc.read::<u32>(Address(ptr)).ok()? as u64;

            let ptr = proc.read::<u32>(Address(ptr)).ok()? as u64;
            Some(Address(ptr))
        }
    } else if core == SUPPORTED_CORES[2] {
        // Swanstation
        if is_64_bit {
            const SIG: Signature<15> = Signature::new("48 89 0D ?? ?? ?? ?? 89 35 ?? ?? ?? ?? 89 3D");
            let addr = SIG.scan_process_range(proc, core_address, proc.get_module_size(core).ok()?)?.0 as i64 + 3;
            let ptr = addr + 0x4 + proc.read::<i32>(Address(addr as u64)).ok()? as i64;
        
            let wram = proc.read::<u64>(Address(ptr as u64)).ok()?;
            Some(Address(wram))
        } else {
            const SIG: Signature<8> = Signature::new("A1 ?? ?? ?? ?? 23 CB 8B");
            let ptr = SIG.scan_process_range(proc, core_address, proc.get_module_size(core).ok()?)?.0 + 1;
            let ptr = proc.read::<u32>(Address(ptr)).ok()? as u64;

            let ptr = proc.read::<u32>(Address(ptr)).ok()? as u64;
            Some(Address(ptr))        }
    } else if core == SUPPORTED_CORES[3] {
        // PCSX ReARMed
        if is_64_bit {
            const SIG: Signature<9> = Signature::new("48 8B 35 ?? ?? ?? ?? 81 E2");
            let addr = SIG.scan_process_range(proc, core_address, proc.get_module_size(core).ok()?)?.0 as i64 + 3;
            let ptr = addr + 0x4 + proc.read::<i32>(Address(addr as u64)).ok()? as i64;
            let ptr = proc.read::<u64>(Address(ptr as u64)).ok()?;
            
            let wram = proc.read::<u64>(Address(ptr)).ok()?;
            Some(Address(wram))
        } else {
            const SIG: Signature<9> = Signature::new("FF FF 1F 00 89 ?? ?? ?? A1");
            let ptr = SIG.scan_process_range(proc, core_address, proc.get_module_size(core).ok()?)?.0 + 9;
            let ptr = Address(proc.read::<u32>(Address(ptr)).ok()? as u64);

            let ptr = Address(proc.read::<u32>(ptr).ok()? as u64);
            Some(ptr)
        }        
    } else {
        None
    }
}

pub fn keep_alive(game: &super::ProcessInfo) -> bool {
    let static_data = STATICDATA.lock();
    game.emulator_process.read::<u8>(static_data.core_addr).is_ok()
}