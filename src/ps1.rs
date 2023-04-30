use asr::{Address, Process};
use bytemuck::CheckedBitPattern;
mod epsxe;
mod xebra;
mod pcsx_redux;
mod duckstation;
mod psxfin;
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
}

impl ProcessInfo {
    fn attach_process() -> Option<Self> {
        let (emulator_type, Some(emulator_process)) = PROCESS_NAMES.iter().map(|name| (name.1, Process::attach(name.0))).find(|p| p.1.is_some())? else { return None };

        Some(Self {
            emulator_type,
            emulator_process,
            wram_base: None,
        })
    }

    fn look_for_wram(&self) -> Option<Address> {
        let addr = match self.emulator_type {
            Emulator::Epsxe => epsxe::epsxe(self),
            Emulator::PsxFin => psxfin::psxfin(self),
            Emulator::Duckstation => duckstation::duckstation(self),
            Emulator::Retroarch => retroarch::retroarch(self),
            Emulator::PcsxRedux => pcsx_redux::pcsx_redux(self),
            Emulator::Xebra => xebra::xebra(self),
        };

        if addr.is_some() {
            asr::set_tick_rate(120.0);
        } else {
            asr::set_tick_rate(0.4);
        }

        addr
    }

    fn keep_alive(&mut self) -> bool {
        match self.emulator_type {
            Emulator::Epsxe => epsxe::keep_alive(),
            Emulator::PsxFin => psxfin::keep_alive(),
            Emulator::Duckstation => duckstation::keep_alive(self),
            Emulator::Retroarch => retroarch::keep_alive(self),
            Emulator::PcsxRedux => pcsx_redux::keep_alive(self),
            Emulator::Xebra => xebra::keep_alive(),
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
/// - ePSXe
/// - pSX
/// - Duckstation
/// - Retroarch (supported cores: Beetle-PSX, Swanstation, PCSX ReARMed)
/// - PCSX-redux
/// - XEBRA
pub fn update() -> bool {
    let state = &mut STATE.lock();
    state.init()
}

/// Reads any value from the emulated RAM.
/// 
/// In PS1, memory addresses are usually mapped at fixed locations starting from `0x80000000`,
/// and is the way many emulators, as well as the GameShark on original hardware, access memory.
/// 
/// For this reason, this method will automatically convert offsets provided in such format.
/// For example providing an offset of `0x1234` or `0x80001234` will return the same value.
/// 
/// Providing any offset outside the range of the PS1's RAM will return `Err()`.
pub fn read<T: CheckedBitPattern>(offset: u32) -> Result<T, asr::Error> {
    if (offset > 0x1FFFFF && offset < 0x80000000) || offset > 0x801FFFFF {
        return Err(asr::Error)
    }; 

    let state = STATE.lock();

    let Some(proc) = &state.proc else {
        return Err(asr::Error)
    };

    let Some(wram) = &proc.wram_base else {
        return Err(asr::Error)
    };

    const WRAMB: u32 = 0x80000000;

    let mut offsetx = offset;
    
    if offsetx >= WRAMB {
        offsetx -= WRAMB
    }

    proc.emulator_process.read(Address(wram.0 + offsetx as u64))
}

#[derive(Copy, Clone, PartialEq)]
enum Emulator {
    Epsxe,
    PsxFin,
    Duckstation,
    Retroarch,
    PcsxRedux,
    Xebra,
}

const PROCESS_NAMES: [(&str, Emulator); 7] = [
    ("ePSXe.exe", Emulator::Epsxe),
    ("psxfin.exe", Emulator::PsxFin),
    ("duckstation-qt-x64-ReleaseLTCG.exe", Emulator::Duckstation),
    ("duckstation-nogui-x64-ReleaseLTCG.exe", Emulator::Duckstation),
    ("retroarch.exe", Emulator::Retroarch),
    ("pcsx-redux.main", Emulator::PcsxRedux),
    ("XEBRA.EXE", Emulator::Xebra),
];