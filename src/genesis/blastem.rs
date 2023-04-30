use asr::{Address, signature::Signature, MemoryRangeFlags};
use super::Endianness;

pub fn blastem(game: &mut super::ProcessInfo) -> Option<Address> {
    const SIG: Signature<16> = Signature::new("72 0E 81 E1 FF FF 00 00 66 8B 89 ?? ?? ?? ?? C3");
    let proc = &game.emulator_process;
    game.endianess = Endianness::LittleEndian;

    let scanned_address = proc.memory_ranges()
        .filter(|m| m.flags().unwrap_or_default().contains(MemoryRangeFlags::WRITE) && m.size().unwrap_or_default() == 0x101000)
        .find_map(|m| SIG.scan_process_range(proc, m.address().unwrap_or_else(|_| Address(0)), m.size().unwrap_or_default()))?
        .0 + 11;

    let wram = proc.read::<u32>(Address(scanned_address)).ok()?;

    Some(Address(wram as u64))
}

pub fn keep_alive() -> bool {
    true
}