use core::fmt::Error;
use asr::{
    Address, Process,
    sync::Mutex,
    primitives::dynamic_endian::{FromEndian, Endian},
};
use bytemuck::CheckedBitPattern;
mod segaclassics;
mod fusion;
mod gens;
mod blastem;
mod retroarch;

static STATE: Mutex<State> = Mutex::new(State {
    proc: None,
});

struct State {
    proc: Option<ProcessInfo>,
}

pub struct ProcessInfo {
    emulator_type: Emulator,
    emulator_process: Process,
    wram_base: Option<Address>,
    endianness: Endian,
}

impl ProcessInfo {
    fn attach_process() -> Option<Self> {
        let (emulator_type, Some(emulator_process)) = PROCESS_NAMES.iter()
            .map(|name| (name.1, Process::attach(name.0)))
            .find(|p| p.1.is_some())? else { return None };

        Some(Self {
            emulator_type,
            emulator_process,
            wram_base: None,
            endianness: Endian::Little,  // Endianness is supposed to be Little, until stated otherwise in the code
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
            Emulator::SegaClassics => segaclassics::keep_alive(self),
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
/// The offset provided must not be higher than `0xFFFF`, otherwise this method will immediately return `Err()`.
///
/// This call is meant to be used by experienced users.
pub fn read_ignoring_endianness<T: CheckedBitPattern>(offset: u32) -> Result<T, Error> {
    if offset > 0xFFFF {
        return Err(Error)
    }

    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(Error) };
    let Some(wram) = &proc.wram_base else { return Err(Error) };
    
    if let Ok(output) = proc.emulator_process.read::<T>(Address(wram.0 + offset as u64)) {
        Ok(output)
    } else {
        Err(Error)
    }
}

/// Reads any value from the emulated RAM.
/// 
/// The offset provided is meant to be the same used on the original, big-endian system.
/// The call will automatically convert the offset and the output value to little endian.
/// 
/// The offset provided must not be higher than `0xFFFF`, otherwise this method will immediately return `Err()`.
pub fn read<T: CheckedBitPattern + FromEndian>(offset: u32) -> Result<T, Error> {
    if (offset > 0xFFFF && offset <= 0xFF0000) || offset > 0xFFFFFF {
        return Err(Error)
    }

    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(Error) };
    let Some(wram) = &proc.wram_base else { return Err(Error) };

    let mut end_offset = offset;

    // Byte swap the offset if needed
    if proc.endianness == Endian::Little && core::mem::size_of::<T>() == 1 {
        if end_offset & 1 == 0 {
            end_offset += 1
        } else {
            end_offset -= 1
        }
    }

    let Ok(value) = proc.emulator_process.read::<T>(Address(wram.0 + end_offset as u64)) else { return Err(Error) };
    Ok(value.from_endian(proc.endianness))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Emulator {
    Retroarch,
    SegaClassics,
    Fusion,
    Gens,
    BlastEm,
}

const PROCESS_NAMES: [(&str, Emulator); 6] = [
    ("retroarch.exe", Emulator::Retroarch),
    ("SEGAGameRoom.exe", Emulator::SegaClassics),
    ("SEGAGenesisClassics.exe", Emulator::SegaClassics),
    ("Fusion.exe", Emulator::Fusion),
    ("gens.exe", Emulator::Gens),
    ("blastem.exe", Emulator::BlastEm),
];