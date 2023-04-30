use asr::{Address, Process};
use bytemuck::CheckedBitPattern;
mod segaclassics;
mod fusion;
mod gens;
mod blastem;
mod retroarch;

static STATE: spinning_top::Spinlock<State> = spinning_top::const_spinlock(State {
    proc: None,
});

struct State {
    proc: Option<ProcessInfo>,
}

pub struct ProcessInfo {
    emulator_type: Emulator,
    emulator_process: Process,
    wram_base: Option<Address>,
    endianess: Endianness,
}

impl ProcessInfo {
    fn attach_process() -> Option<Self> {
        let (emulator_type, Some(emulator_process)) = PROCESS_NAMES.iter().map(|name| (name.1, Process::attach(name.0))).find(|p| p.1.is_some())? else { return None };

        Some(Self {
            emulator_type,
            emulator_process,
            wram_base: None,
            endianess: Endianness::LittleEndian,
        })
    }

    fn look_for_wram(&mut self) -> Option<Address> {
        match self.emulator_type {
            Emulator::Retroarch => retroarch::retroarch(self),
            Emulator::SegaClassics => segaclassics::segaclassics(self),
            Emulator::Fusion => fusion::fusion(self),
            Emulator::Gens => gens::gens(self),
            Emulator::BlastEm => blastem::blastem(self),
        }      
    }

    fn keep_alive(&mut self) -> bool {
        match self.emulator_type {
            Emulator::Retroarch => retroarch::keep_alive(self),
            Emulator::SegaClassics => segaclassics::keep_alive(),
            Emulator::Fusion => fusion::keep_alive(self),
            Emulator::Gens => gens::keep_alive(),
            Emulator::BlastEm => blastem::keep_alive(),
        }      
    }
}

impl State {
    fn init(&mut self) -> bool {
        if self.proc.is_none() {
            self.proc = ProcessInfo::attach_process()
        }

        let Some(game) = &mut self.proc else {
            return false
        };

        if !game.emulator_process.is_open() {
            self.proc = None;
            return false
        }

        if game.wram_base.is_none() {
            game.wram_base = game.look_for_wram();
            
            if game.wram_base.is_none() {
                return false
            }
        }

        if !game.keep_alive() {
            game.wram_base = None
        }

        game.wram_base.is_some()
    }
}

/// Calls the internal routines needed in order to hook to the target emulator and find the address of the emulated RAM.
/// 
/// Returns true if successful, false otherwise.
/// 
/// Supported emulators are:
/// - Retroarch
/// - SEGA Classics / SEGA Game Room
/// - Fusion
/// - Gens
/// - BlastEm
pub fn update() -> bool {
    let state = &mut STATE.lock();
    state.init()
}

/// Reads  raw data from the emulated RAM ignoring all endianess settings
/// The same call, performed on two different emulators, can be different
/// due to the endianness used by the emulator.
/// 
/// This call is meant to be used by experienced users.
pub fn read_ignoring_endianness<T: CheckedBitPattern>(offset: u32) -> Result<T, asr::Error> {
    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(asr::Error) };
    let Some(wram) = &proc.wram_base else { return Err(asr::Error) };
    proc.emulator_process.read(Address(wram.0 + offset as u64))
}

/// Read an u8 from the emulated RAM.
/// 
/// The offset provided is meant to be the same used on the original, big-endian system.
/// The call will automatically convert the offset and the output value to little endian. 
pub fn read_u8(offset: u32) -> Result<u8, asr::Error> {
    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(asr::Error) };
    let Some(wram) = &proc.wram_base else { return Err(asr::Error) };

    let mut end_offset = offset;
    if proc.endianess == Endianness::LittleEndian {
        if end_offset & 1 == 0 {
            end_offset += 1
        } else {
            end_offset -= 1
        }
    }

    proc.emulator_process.read::<u8>(Address(wram.0 + end_offset as u64))
}

/// Read an i8 from the emulated RAM.
/// 
/// The offset provided is meant to be the same used on the original, big-endian system.
/// The call will automatically convert the offset and the output value to little endian. 
pub fn read_i8(offset: u32) -> Result<i8, asr::Error> {
    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(asr::Error) };
    let Some(wram) = &proc.wram_base else { return Err(asr::Error) };

    let mut end_offset = offset;
    if proc.endianess == Endianness::LittleEndian {
        if end_offset & 1 == 0 {
            end_offset += 1
        } else {
            end_offset -= 1
        }
    }

    proc.emulator_process.read::<i8>(Address(wram.0 + end_offset as u64))
}

/// Read an u16 from the emulated RAM.
/// 
/// The offset provided is meant to be the same used on the original, big-endian system.
/// The call will automatically convert the offset and the output value to little endian. 
pub fn read_u16(offset: u32) -> Result<u16, asr::Error> {
    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(asr::Error) };
    let Some(wram) = &proc.wram_base else { return Err(asr::Error) };

    let value = proc.emulator_process.read::<u16>(Address(wram.0 + offset as u64));

    if let Ok(t_value) = value {
        if proc.endianess == Endianness::BigEndian {
            Ok(u16::from_be(t_value))
        } else {
            value
        }
    } else {
        value
    }
}

/// Read an i16 from the emulated RAM.
/// 
/// The offset provided is meant to be the same used on the original, big-endian system.
/// The call will automatically convert the offset and the output value to little endian. 
pub fn read_i16(offset: u32) -> Result<i16, asr::Error> {
    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(asr::Error) };
    let Some(wram) = &proc.wram_base else { return Err(asr::Error) };

    let value = proc.emulator_process.read::<i16>(Address(wram.0 + offset as u64));

    if let Ok(t_value) = value {
        if proc.endianess == Endianness::BigEndian {
            Ok(i16::from_be(t_value))
        } else {
            value
        }
    } else {
        value
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Emulator {
    Retroarch,
    SegaClassics,
    Fusion,
    Gens,
    BlastEm,
}

#[derive(PartialEq)]
enum Endianness {
    LittleEndian,
    BigEndian,
}

const PROCESS_NAMES: [(&str, Emulator); 6] = [
    ("retroarch.exe", Emulator::Retroarch),
    ("SEGAGameRoom.exe", Emulator::SegaClassics),
    ("SEGAGenesisClassics.exe", Emulator::SegaClassics),
    ("Fusion.exe", Emulator::Fusion),
    ("gens.exe", Emulator::Gens),
    ("blastem.exe", Emulator::BlastEm),
];