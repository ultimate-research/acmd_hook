#![feature(proc_macro_hygiene)]

use smash::lib::L2CValue;
use smash::lua2cpp::L2CFighterCommon;
use smash::lua2cpp::L2CAgentBase;
use smash::phx::Hash40;
use parking_lot::RwLock;

type Callback = fn(&mut L2CFighterCommon);
type Predicate = unsafe fn(&mut L2CAgentBase, Hash40) -> bool;

static HOOKS: RwLock<Vec<Callback>> = RwLock::new(Vec::new());
static PREDS: RwLock<Vec<Predicate>> = RwLock::new(Vec::new());

#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_sys_line_system_control_fighter)]
pub unsafe fn sys_line_system_control_fighter_hook(fighter: &mut L2CFighterCommon) -> L2CValue {
    for hook in HOOKS.read().iter() {
        hook(fighter)
    }

    original!()(fighter)
}

#[skyline::hook(replace = smash::lua2cpp::L2CAgentBase_call_coroutine)]
pub unsafe fn call_coroutine_hook(
    agent: &mut smash::lua2cpp::L2CAgentBase, 
    index: i32, 
    hash: smash::phx::Hash40
) -> u64 {
    for pred in PREDS.read().iter() {
        if pred(agent, hash) {
            return 0;
        }
    }

    original!()(agent, index, hash)
}


fn nro_main(nro: &skyline::nro::NroInfo) {
    match nro.name {
        "common" => {
            skyline::install_hook!(sys_line_system_control_fighter_hook);
            skyline::install_hook!(call_coroutine_hook);
        }
        _ => (),
    }
}

#[skyline::main(name = "acmd_hook")]
pub fn main() {
    skyline::nro::add_hook(nro_main).unwrap();
}

#[no_mangle]
pub extern "Rust" fn add_acmd_load_hook(hook: Callback, predicate: Predicate) {
    let mut hooks = HOOKS.write();
    let mut preds = PREDS.write();

    hooks.push(hook);
    preds.push(predicate);
}
