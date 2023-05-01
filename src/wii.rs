use core::fmt::Error;
use asr::{
    Address, Process,
    sync::Mutex,
    primitives::dynamic_endian::{FromEndian, Endian},
};
use bytemuck::CheckedBitPattern;
mod dolphin;

static STATE: Mutex<State> = Mutex::new(State {
    proc: None,
});

struct State {
    proc: Option<ProcessInfo>,
}

pub struct ProcessInfo {
    emulator_type: Emulator,
    emulator_process: Process,
    mem_1: Option<Address>,
    mem_2: Option<Address>,
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
            mem_1: None,
            mem_2: None,
            endianness: Endian::Big,  // The only emulator worth mentioning for the Wii (Dolphin), uses Big Endian
        })
    }

    fn look_for_wram(&mut self) -> (Option<Address>, Option<Address>) {
        match self.emulator_type {
            Emulator::Dolphin => dolphin::dolphin(self),
        }      
    }

    fn keep_alive(&self) -> bool {
        match self.emulator_type {
            Emulator::Dolphin => dolphin::keep_alive(self),
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

        if game.mem_1.is_none() || game.mem_2.is_none() {
            (game.mem_1, game.mem_2) = game.look_for_wram();
            
            if game.mem_1.is_none() || game.mem_2.is_none() {
                return false
            }
        }

        if !game.keep_alive() {
            game.mem_1 = None;
            game.mem_2 = None;
        }

        game.mem_1.is_some() && game.mem_2.is_some()
    }
}

/// Calls the internal routines needed in order to hook to the target emulator and find the address of the emulated RAM.
/// 
/// Returns true if successful, false otherwise.
/// 
/// As of now, the only supported emulator is Dolphin.
pub fn update() -> bool {
    let state = &mut STATE.lock();
    state.init()
}

/// Reads any value from the emulated RAM.
/// 
/// The address provided is meant to be the mapped address used on the original, big-endian system.
/// The call will automatically convert the address provided to its corresponding offset from
/// `MEM1` or `MEM2` and read the value, converting it to little endian if necessary.
/// 
/// The address provided has to match a mapped memory address on the original Wii:
/// - Valid addresses for `MEM1` range from `0x80000000` to `0x817FFFFF`
/// - Valid addresses for `MEM2` range from `0x90000000` to `0x93FFFFFF`
/// 
/// Values below and up to `0x017FFFFF` are assumed to be offsets from `MEM1`'s base address.
/// Any other invalid value will make this method immediately return `Err()`.
pub fn read<T: CheckedBitPattern + FromEndian>(address: u32) -> Result<T, Error> {
    if (address > 0x017FFFFF && address < 0x80000000)
        || (address > 0x817FFFFF && address < 0x90000000)
        || address > 0x93FFFFFF {
        return Err(Error)
    }

    if address <= 0x017FFFFF || (address >= 0x80000000 && address <= 0x817FFFFF)
    {
        read_from_mem_1(address)
    } else if address >= 0x90000000 && address <= 0x93FFFFFF {
        read_from_mem_2(address)
    } else {
        Err(Error)
    }
}

/// Reads any value from `MEM1`.
/// 
/// The address provided is meant to be the mapped address used on the original, big-endian system.
/// The call will automatically convert the address provided to its corresponding offset from
/// `MEM1` and read the value, converting it to little endian if necessary.
/// 
/// The address provided has to match a mapped memory address on the original Wii. 
/// Valid addresses for `MEM1` range from `0x80000000` to `0x817FFFFF`
/// 
/// Values below and up to `0x017FFFFF` are assumed to be offsets from `MEM1`'s base address.
/// Any other invalid value will make this method immediately return `Err()`.
pub fn read_from_mem_1<T: CheckedBitPattern + FromEndian>(address: u32) -> Result<T, Error> {
    const SHIFT: u32 = 0x80000000;

    if (address > 0x017FFFFF && address < SHIFT) || address > 0x817FFFFF {
        return Err(Error)
    }

    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(Error) };
    let Some(mem_1) = &proc.mem_1 else { return Err(Error) };

    let mut offset = address;

    if offset >= SHIFT {
        offset -= SHIFT
    }

    let Ok(value) = proc.emulator_process.read::<T>(Address(mem_1.0 + offset as u64)) else { return Err(Error) };
    Ok(value.from_endian(proc.endianness))
}

/// Reads any value from `MEM2`.
/// 
/// The address provided is meant to be the mapped address used on the original, big-endian system.
/// The call will automatically convert the address provided to its corresponding offset from
/// `MEM2` and read the value, converting it to little endian if necessary.
/// 
/// The address provided has to match a mapped memory address on the original Wii. 
/// Valid addresses for `MEM2` range from `0x90000000` to `0x93FFFFFF`
/// 
/// Values below and up to `0x03FFFFFF` are assumed to be offsets from `MEM2`'s base address.
/// Any other invalid value will make this method immediately return `Err()`.
pub fn read_from_mem_2<T: CheckedBitPattern + FromEndian>(address: u32) -> Result<T, Error> {
    const SHIFT: u32 = 0x90000000;

    if address < SHIFT || address > 0x93FFFFFF {
        return Err(Error)
    }

    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(Error) };
    let Some(mem_2) = &proc.mem_2 else { return Err(Error) };

    let offset = address - SHIFT;

    let Ok(value) = proc.emulator_process.read::<T>(Address(mem_2.0 + offset as u64)) else { return Err(Error) };
    Ok(value.from_endian(proc.endianness))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Emulator {
    Dolphin,
}

const PROCESS_NAMES: [(&str, Emulator); 1] = [
    ("Dolphin.exe", Emulator::Dolphin),
];