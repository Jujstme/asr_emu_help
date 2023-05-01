use asr::{Address, signature::Signature, MemoryRangeFlags, sync::Mutex};
use crate::shared::check_for_64_bit;

static STATICDATA: Mutex<StaticData> = Mutex::new(StaticData {
    is_64_bit: false,
    addr_base: 0,
    addr: 0,
});

struct StaticData {
    is_64_bit: bool,
    addr_base: u64,
    addr: u64,
}

pub fn pcsx_redux(game: &super::ProcessInfo) -> Option<Address> {
    let proc = &game.emulator_process;

    let (Some(main_module_address), Some(main_module_size)) = super::PROCESS_NAMES.iter()
        .filter(|p| p.1 == super::Emulator::PcsxRedux)
        .map(|m| (proc.get_module_address(m.0).ok(), proc.get_module_size(m.0).ok()))
        .find(|m| m.0.is_some() && m.1.is_some())? else { return None };
    
    let mut static_data = STATICDATA.lock();
    let is_64_bit = check_for_64_bit(proc, main_module_address);
    static_data.is_64_bit = is_64_bit;

    if is_64_bit {
        const SIG_BASE: Signature<25> = Signature::new("48 B9 ?? ?? ?? ?? ?? ?? ?? ?? E8 ?? ?? ?? ?? C7 85 ?? ?? ?? ?? 00 00 00 00");
        const SIG_OFFSET: Signature<9> = Signature::new("89 D1 C1 E9 10 48 8B ?? ??");

        let addr_base = SIG_BASE.scan_process_range(proc, main_module_address, main_module_size)?.0 + 2;
        static_data.addr_base = addr_base;
        let addr_base = proc.read::<u64>(Address(addr_base)).ok()?;
        static_data.addr = addr_base;

        let offset = SIG_OFFSET.scan_process_range(proc, main_module_address, main_module_size)?.0 + 8;
        let offset = proc.read::<u8>(Address(offset)).ok()? as u64;
        
        let addr = proc.read::<u64>(Address(addr_base + offset)).ok()?;
        let addr = proc.read::<u64>(Address(addr)).ok()?;
        
        Some(Address(addr))
    } else {
        const SIG: Signature<18> = Signature::new("8B 3D 20 ?? ?? ?? 0F B7 D3 8B 04 95 ?? ?? ?? ?? 21 05");

        let addr_base = proc.memory_ranges()
            .filter(|m| m.flags().unwrap_or_default().contains(MemoryRangeFlags::WRITE))
            .find_map(|m| SIG.scan_process_range(proc, m.address().unwrap_or_else(|_| Address(0)), m.size().unwrap_or_default()))?
            .0 + 2;
        
        static_data.addr_base = addr_base;

        let addr = proc.read::<u32>(Address(addr_base)).ok()? as u64;
        static_data.addr = addr;

        Some(Address(addr))
    }
}

pub fn keep_alive(game: &super::ProcessInfo) -> bool {
    let static_data = STATICDATA.lock();
    
    if static_data.addr_base == 0 {
        return false
    }

    if static_data.is_64_bit {
        Some(static_data.addr) == game.emulator_process.read::<u64>(Address(static_data.addr_base)).ok()
    } else {
        let Some(addr) = game.emulator_process.read::<u32>(Address(static_data.addr_base)).ok() else { return false };
        static_data.addr == addr as u64
    }
}