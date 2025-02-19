
use super::database_mgr::DatabaseMgr;
use super::script_mgr::ScriptMgr;
use super::window_mgr::WindowMgr;
use super::EmulatorState;

use crate::dbobjects::calculation::context::CalculationContext;

pub struct EmulatorContext<'a> {
    pub database_mgr: &'a DatabaseMgr,
    pub variables: &'a Vec<(String, String)>,
    pub globals: &'a Vec<(String, String)>,
    pub window_mgr: &'a WindowMgr,
    pub state: &'a EmulatorState,
}

impl CalculationContext for EmulatorContext<'_> {
    fn get_record_id() -> u32 { todo!() }
    fn get_account_name() -> String { todo!() }
    fn get_active_field_contents() -> String { todo!() }
    fn get_active_fieldname() -> String { todo!() }
    fn get_host_ip_addr() -> String { todo!() }
}
