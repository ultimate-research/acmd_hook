#![feature(proc_macro_hygiene)]

use smash::lib::L2CValue;
use smash::lua2cpp::{L2CAgentBase, L2CFighterBase, L2CFighterCommon};
use smash::phx::Hash40;
use parking_lot::RwLock;

type Callback = fn(&mut L2CFighterCommon);
type WeaponCallback = fn(&mut L2CFighterBase);
type Predicate = unsafe fn(&mut L2CAgentBase, Hash40) -> bool;

static HOOKS: RwLock<Vec<Callback>> = RwLock::new(Vec::new());
static WEAPON_HOOKS: RwLock<Vec<WeaponCallback>> = RwLock::new(Vec::new());
static PREDS: RwLock<Vec<Predicate>> = RwLock::new(Vec::new());
static WEAPON_PREDS: RwLock<Vec<Predicate>> = RwLock::new(Vec::new());
static PRED_HOOK_INDEXES: RwLock<Vec<usize>> = RwLock::new(Vec::new());
static PRED_WEAPON_HOOK_INDEXES: RwLock<Vec<usize>> = RwLock::new(Vec::new());

#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_sys_line_system_control_fighter)]
pub unsafe fn sys_line_system_control_fighter_hook(fighter: &mut L2CFighterCommon) -> L2CValue {
    for hook in HOOKS.read().iter() {
        hook(fighter)
    }

    original!()(fighter)
}

#[skyline::hook(replace = smash::lua2cpp::L2CFighterBase_sys_line_system_control)]
pub unsafe fn sys_line_system_control_hook(fighter_base: &mut L2CFighterBase) -> L2CValue {
    for hook in WEAPON_HOOKS.read().iter() {
        hook(fighter_base)
    }

    original!()(fighter_base)
}

#[skyline::hook(replace = smash::lua2cpp::L2CAgentBase_call_coroutine)]
pub unsafe fn call_coroutine_hook(
    agent: &mut smash::lua2cpp::L2CAgentBase, 
    index: i32, 
    hash: smash::phx::Hash40
) -> u64 {
    let hooks = HOOKS.read();
    let preds = PREDS.read();
    let pred_indexes = PRED_HOOK_INDEXES.read();
    
    let weapon_hooks = WEAPON_HOOKS.read();
    let weapon_preds = WEAPON_PREDS.read();
    let weapon_pred_indexes = PRED_WEAPON_HOOK_INDEXES.read();
    for i in 0..preds.len() {
        let pred = preds[i];
        if pred(agent, hash) {
            let hook = hooks[pred_indexes[i]];
            let mut fighter_common = L2CFighterCommon{
                fighter_base: L2CFighterBase{
                    agent_base: *agent,
                    global_table: L2CValue::new_void()
                }
            };
            hook(&mut fighter_common);
            return 0;
        }
    }

    for i in 0..weapon_preds.len() {
        let pred = weapon_preds[i];
        if pred(agent, hash) {
            let hook = weapon_hooks[weapon_pred_indexes[i]];
            let mut fighter_base = L2CFighterBase{
                agent_base: *agent,
                global_table: L2CValue::new_void()
            };
            hook(&mut fighter_base);
            return 0;
        }
    }

    original!()(agent, index, hash)
}


fn nro_main(nro: &skyline::nro::NroInfo) {
    match nro.name {
        "common" => {
            skyline::install_hooks!(
                sys_line_system_control_hook,
                sys_line_system_control_fighter_hook,
                call_coroutine_hook);
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
    let mut pred_indexes = PRED_HOOK_INDEXES.write();

    hooks.push(hook);
    preds.push(predicate);
    pred_indexes.push(hooks.len() - 1);
}

#[no_mangle]
pub extern "Rust" fn add_acmd_load_weapon_hook(hook: WeaponCallback, predicate: Predicate) {
    let mut weapon_hooks = WEAPON_HOOKS.write();
    let mut weapon_preds = WEAPON_PREDS.write();
    let mut weapon_pred_indexes = PRED_WEAPON_HOOK_INDEXES.write();

    weapon_hooks.push(hook);
    weapon_preds.push(predicate);
    weapon_pred_indexes.push(weapon_hooks.len() - 1);
}
