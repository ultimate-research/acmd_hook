#![feature(proc_macro_hygiene)]

use smash::lib::L2CValue;
use smash::lua2cpp::L2CFighterCommon;
use parking_lot::RwLock;

type Callback = fn(&mut L2CFighterCommon);

static HOOKS: RwLock<Vec<Callback>> = RwLock::new(Vec::new());

#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_sys_line_system_control_fighter)]
pub unsafe fn sys_line_system_control_fighter_hook(fighter: &mut L2CFighterCommon) -> L2CValue {
    for hook in HOOKS.read().iter() {
        hook(fighter)
    }

    original!()(fighter)
}


fn nro_main(nro: &skyline::nro::NroInfo) {
    match nro.name {
        "common" => {
            skyline::install_hook!(sys_line_system_control_fighter_hook);
        }
        _ => (),
    }
}

#[skyline::main(name = "acmd_hook")]
pub fn main() {
    skyline::nro::add_hook(nro_main).unwrap();
}

#[no_mangle]
pub extern "Rust" fn add_acmd_load_hook(callback: Callback) {
    let mut hooks = HOOKS.write();

    hooks.push(callback);
}
