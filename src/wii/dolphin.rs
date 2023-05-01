use asr::{Address, primitives::dynamic_endian::Endian};

pub fn dolphin(game: &mut super::ProcessInfo) -> (Option<Address>, Option<Address>) {
    let proc = &game.emulator_process;
    game.endianness = Endian::Big;

    let mut mem_1_addr: u64 = 0;
    let mut mem_2_addr: u64 = 0;

    // Adapted scanning code from Dolphin Memory Engine
    for entry in proc.memory_ranges() {
        let size = entry.size().unwrap_or_default();

        // MEM2 address
        // MEM2 is found thanks to its fixed size
        if mem_2_addr == 0 && size == 0x4000000 {
            // If we reached a region that is too far away from MEM1, this will exit the loop immediately
            if mem_1_addr != 0 && entry.address().unwrap_or_else(|_| Address(0)).0 > mem_1_addr + 0x10000000 {
                break;
            }

            mem_2_addr = entry.address().unwrap_or_else(|_| Address(0)).0;

        } else if size == 0x2000000 {
            // If we find any MEM1 region but not a MEM2 region close to it, this will ensure
            // the loop continues looking for other possible MEM1 regions
            mem_1_addr = entry.address().unwrap_or_else(|_| Address(0)).0;

        }
        
        // This code really should be never run, but it's implemented as a failsafe.
        // If both MEM1 and MEM2 are found, break out of the loop immediately
        if mem_1_addr != 0 && mem_2_addr != 0 {
            break;
        }
    }

    
    if mem_1_addr == 0 || mem_2_addr == 0 {
        (None, None)
    } else {
        (Some(Address(mem_1_addr)), Some(Address(mem_2_addr)))
    }

}

pub fn keep_alive(game: &super::ProcessInfo) -> bool {
    let Some(mem_1) = &game.mem_1 else { return false };
    let Some(mem_2) = &game.mem_2 else { return false };

    game.emulator_process.read::<u8>(*mem_1).is_ok() && game.emulator_process.read::<u8>(*mem_2).is_ok()
}