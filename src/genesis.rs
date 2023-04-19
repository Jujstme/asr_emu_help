use asr::{Address, Process};
use bytemuck::Pod;
mod segaclassics;
mod fusion;
mod gens;

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
    endianess: Endianess,
}

impl ProcessInfo {
    fn attach_process() -> Option<Self> {
        const PROCESS_NAMES: [&str; 7] = [
            "retroarch.exe",
            "SEGAGameRoom.exe",
            "SEGAGenesisClassics.exe",
            "Fusion.exe",
            "gens.exe",
            "blastem.exe",
            "EmuHawk.exe",
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
            "retroarch.exe" => Emulator::Retroarch,
            "SEGAGameRoom.exe" | "SEGAGenesisClassics.exe" => Emulator::SegaClassics,
            "Fusion.exe" => Emulator::Fusion,
            "gens.exe" => Emulator::Gens,
            "blastem.exe" => Emulator::BlastEm,
            "EmuHawk.exe" => Emulator::EmuHawk,
            _ => return None,
        };

        Some(Self {
            emulator_type,
            emulator_process,
            wram_base: None,
            endianess: Endianess::LittleEndian,
        })
    }

    fn look_for_wram(&mut self) -> Option<Address> {
        match self.emulator_type {
            //Emulator::Retroarch,
            Emulator::SegaClassics => segaclassics::segaclassics(self),
            Emulator::Fusion => fusion::fusion(self),
            Emulator::Gens => gens::gens(self),
            // Emulator::BlastEm => blastem::blastem(self),
            //EmuHawk,
            _ => None,
        }      
    }

    fn keep_alive(&mut self) -> bool {
        match self.emulator_type {
            //Emulator::Retroarch,
            Emulator::SegaClassics => segaclassics::keep_alive(),
            Emulator::Fusion => fusion::keep_alive(),
            Emulator::Gens => gens::keep_alive(),
            // Emulator::BlastEm => blastem::keep_alive(),
            //EmuHawk,
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

pub fn read_ignoring_endianess<T: Pod>(offset: u32) -> Result<T, asr::Error> {
    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(asr::Error) };
    let Some(wram) = &proc.wram_base else { return Err(asr::Error) };
    proc.emulator_process.read(Address(wram.0 + offset as u64))
}

pub fn read_u8(offset: u32) -> Result<u8, asr::Error> {
    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(asr::Error) };
    let Some(wram) = &proc.wram_base else { return Err(asr::Error) };

    let mut end_offset = offset;
    if proc.endianess == Endianess::LittleEndian {
        if end_offset & 1 == 0 {
            end_offset += 1
        } else {
            end_offset -= 1
        }
    }

    proc.emulator_process.read::<u8>(Address(wram.0 + end_offset as u64))
}

pub fn read_i8(offset: u32) -> Result<i8, asr::Error> {
    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(asr::Error) };
    let Some(wram) = &proc.wram_base else { return Err(asr::Error) };

    let mut end_offset = offset;
    if proc.endianess == Endianess::LittleEndian {
        if end_offset & 1 == 0 {
            end_offset += 1
        } else {
            end_offset -= 1
        }
    }

    proc.emulator_process.read::<i8>(Address(wram.0 + end_offset as u64))
}

pub fn read_u16(offset: u32) -> Result<u16, asr::Error> {
    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(asr::Error) };
    let Some(wram) = &proc.wram_base else { return Err(asr::Error) };

    let value = proc.emulator_process.read::<u16>(Address(wram.0 + offset as u64));

    if let Ok(t_value) = value {
        if proc.endianess == Endianess::BigEndian {
            Ok(u16::from_be(t_value))
        } else {
            value
        }
    } else {
        value
    }
}

pub fn read_i16(offset: u32) -> Result<i16, asr::Error> {
    let state = STATE.lock();
    let Some(proc) = &state.proc else { return Err(asr::Error) };
    let Some(wram) = &proc.wram_base else { return Err(asr::Error) };

    let value = proc.emulator_process.read::<i16>(Address(wram.0 + offset as u64));

    if let Ok(t_value) = value {
        if proc.endianess == Endianess::BigEndian {
            Ok(i16::from_be(t_value))
        } else {
            value
        }
    } else {
        value
    }
}

enum Emulator {
    Retroarch,
    SegaClassics,
    Fusion,
    Gens,
    BlastEm,
    EmuHawk,
}

#[derive(PartialEq)]
enum Endianess {
    LittleEndian,
    BigEndian,
}