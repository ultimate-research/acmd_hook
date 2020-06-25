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

unsafe extern "C" fn nullify_acmd(arg1: *mut smash::lib::L2CAgent, arg2: *mut skyline::libc::c_void) -> u64 {
    return 0;
}

pub fn get_category(boma: &mut smash::app::BattleObjectModuleAccessor) -> i32{
    return (boma.info >> 28) as u8 as i32;
}
extern "C"{
    #[link_name = "\u{1}_ZN3app7utility8get_kindEPKNS_26BattleObjectModuleAccessorE"]
    pub fn get_kind(module_accessor: &mut smash::app::BattleObjectModuleAccessor) -> i32;
}

#[skyline::hook(replace = smash::lua2cpp::L2CAgentBase_call_coroutine)]
pub unsafe fn call_coroutine_hook(
    agent: *mut smash::lua2cpp::L2CAgentBase, 
    arg1: i32, 
    arg2: smash::phx::Hash40
) -> u64 {
    if arg2.hash == smash::hash40("game_attackairf") {
        let lua_state = (*agent).lua_state_agent;
        let module_accessor = smash::app::sv_system::battle_object_module_accessor(lua_state);
        if get_category(module_accessor) ==  smash::lib::lua_const::BATTLE_OBJECT_CATEGORY_FIGHTER
            && get_kind(module_accessor) == smash::lib::lua_const::FIGHTER_KIND_MARIO {
                return 0;
            }
    }

    original!()(agent, arg1, arg2)
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
pub extern "Rust" fn add_acmd_load_hook(callback: Callback) {
    let mut hooks = HOOKS.write();

    hooks.push(callback);
}
