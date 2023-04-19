use asr::{Address, Process};
use bytemuck::Pod;
mod epsxe;
mod xebra;

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
        const PROCESS_NAMES: [&str; 8] = [
            "ePSXe.exe",
            "psxfin.exe",
            "duckstation-qt-x64-ReleaseLTCG.exe", "duckstation-nogui-x64-ReleaseLTCG.exe",
            "retroarch.exe",
            "pcsx-redux.main",
            "XEBRA.EXE",
            "EmuHawk.exe"
        ];

        let mut proc: Option<Process> = None;
        let mut proc_name: Option<&str> = None;

        for name in PROCESS_NAMES {
            proc = Process::attach(name);
            if proc.is_some() {
                proc_name = Some(name);
                break
            }
        }

        let emulator_process = proc?;

        let emulator_type = match proc_name? {
            "ePSXe.exe" => Emulator::Epsxe,
            //"psxfin.exe" => Emulator::pSX,
            "duckstation-qt-x64-ReleaseLTCG.exe" | "duckstation-nogui-x64-ReleaseLTCG.exe" => Emulator::Duckstation,
            "retroarch.exe" => Emulator::Retroarch,
            //"pcsx-redux.main" => Emulator::PCSX_Redux,
            "XEBRA.EXE" => Emulator::Xebra,
            "EmuHawk.exe" => Emulator::EmuHawk,
            _ => return None,
        };

        Some(Self {
            emulator_type,
            emulator_process,
            wram_base: None,
        })
    }

    fn look_for_wram(&mut self) -> Option<Address> {
        match self.emulator_type {
            Emulator::Epsxe => epsxe::epsxe(self),
            //"psxfin.exe",
            //"duckstation-qt-x64-ReleaseLTCG.exe", "duckstation-nogui-x64-ReleaseLTCG.exe",
            //"retroarch.exe",
            //"pcsx-redux.main",
            Emulator::Xebra => xebra::xebra(self),
            //"EmuHawk.exe"
            _ => None,
        }      
    }

    fn keep_alive(&mut self) -> bool {
        match self.emulator_type {
            Emulator::Epsxe => epsxe::keep_alive(),
            //"psxfin.exe",
            //"duckstation-qt-x64-ReleaseLTCG.exe", "duckstation-nogui-x64-ReleaseLTCG.exe",
            //"retroarch.exe",
            //"pcsx-redux.main",
            Emulator::Xebra => xebra::keep_alive(),
            //"EmuHawk.exe"
            _ => false,
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

pub fn update() -> bool {
    let state = &mut STATE.lock();
    state.init()
}

pub fn read<T: Pod>(offset: u32) -> Result<T, asr::Error> {
    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(asr::Error) };
    let Some(wram) = &proc.wram_base else { return Err(asr::Error) };
    proc.emulator_process.read(Address(wram.0 + offset as u64))
}

enum Emulator {
    Epsxe,
    //pSX,
    Duckstation,
    Retroarch,
    //PCSX_Redux,
    Xebra,
    EmuHawk,
}