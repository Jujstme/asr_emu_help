use asr::{Address, signature::Signature, MemoryRangeFlags};
use super::Endianness;

static STATICDATA: spinning_top::Spinlock<StaticData> = spinning_top::const_spinlock(StaticData {
    core_base: Address(0),
});

struct StaticData {
    core_base: Address,
}

pub fn retroarch(game: &mut super::ProcessInfo) -> Option<Address> {
    const SUPPORTED_CORES: [&str; 4] = [
        "blastem_libretro.dll",
        "genesis_plus_gx_libretro.dll",
        "genesis_plus_gx_wide_libretro.dll",
        "picodrive_libretro.dll",
    ]; 

    let proc = &game.emulator_process;
    let mut static_data = STATICDATA.lock();

    let Some(main_module_address) = super::PROCESS_NAMES.iter()
        .filter(|p| p.1 == super::Emulator::Retroarch)
        .map(|m| proc.get_module_address(m.0).ok())
        .find(|m| m.is_some())? else { return None };

    let is_64_bit = crate::shared::check_for_64_bit(proc, main_module_address);

    let (&core_name, Ok(core_address)) = SUPPORTED_CORES.iter()
        .map(|m| (m, proc.get_module_address(m)))
        .find(|m| m.1.is_ok())? else { return None };

    static_data.core_base = core_address;

    if core_name == SUPPORTED_CORES[0] {
        game.endianess = Endianness::LittleEndian;

        // BlastEm
        const SIG: Signature<16> = Signature::new("72 0E 81 E1 FF FF 00 00 66 8B 89 ?? ?? ?? ?? C3");
        let scanned_address = proc.memory_ranges()
            .filter(|m| m.flags().unwrap_or_default().contains(MemoryRangeFlags::WRITE) && m.size().unwrap_or_default() == 0x101000)
            .find_map(|m| SIG.scan_process_range(proc, m.address().unwrap_or_else(|_| Address(0)), m.size().unwrap_or_default()))?
            .0 + 11;

        let wram = proc.read::<u32>(Address(scanned_address)).ok()?;

        Some(Address(wram as u64))
    } else if core_name == SUPPORTED_CORES[1] || core_name == SUPPORTED_CORES[2] {
        game.endianess = Endianness::LittleEndian;

        // Genesis plus GX
        if is_64_bit {
            const SIG_64: Signature<10> = Signature::new("48 8D 0D ?? ?? ?? ?? 4C 8B 2D");
            let addr = SIG_64.scan_process_range(proc, core_address, proc.get_module_size(core_name).ok()?)?.0 as i64 + 3;
            let wram = addr + 0x4 + proc.read::<i32>(Address(addr as u64)).ok()? as i64;
            Some(Address(wram as u64))
        } else {
            const SIG_32: Signature<7> = Signature::new("A3 ?? ?? ?? ?? 29 F9");
            let ptr = SIG_32.scan_process_range(proc, core_address, proc.get_module_size(core_name).ok()?)?.0 + 1;
            let wram = proc.read::<u32>(Address(ptr)).ok()? as u64;
            Some(Address(wram))        }
    } else if core_name == SUPPORTED_CORES[3] {
        game.endianess = Endianness::LittleEndian;

        // Picodrive
        if is_64_bit {
            const SIG_64: Signature<9> = Signature::new("48 8D 0D ?? ?? ?? ?? 41 B8");
            let addr = SIG_64.scan_process_range(proc, core_address, proc.get_module_size(core_name).ok()?)?.0 as i64 + 3;
            let wram = addr + 0x4 + proc.read::<i32>(Address(addr as u64)).ok()? as i64;
            Some(Address(wram as u64))
        } else {
            const SIG_32: Signature<8> = Signature::new("B9 ?? ?? ?? ?? C1 EF 10");
            let ptr = SIG_32.scan_process_range(proc, core_address, proc.get_module_size(core_name).ok()?)?.0 + 1;
            let wram = proc.read::<u32>(Address(ptr)).ok()? as u64;
            Some(Address(wram))
        }        
    } else {
        None
    }
}

pub fn keep_alive(game: &super::ProcessInfo) -> bool {
    let static_data = STATICDATA.lock();
    game.emulator_process.read::<u8>(static_data.core_base).is_ok()
}