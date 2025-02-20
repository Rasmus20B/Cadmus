
use super::database_mgr::DatabaseMgr;
use crate::dbobjects::reference::FieldReference;
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
    fn get_record_id(&self) -> u32 { todo!() }
    fn get_account_name(&self) -> String { todo!() }
    fn get_active_field_contents(&self) -> String { todo!() }
    fn get_active_fieldname(&self) -> String { todo!() }
    fn get_host_ip_addr(&self) -> String { todo!() }

    fn get_var(&self, name: &str) -> Option<String> { 
        self.variables.iter()
            .find(|var| var.0 == name)
            .map(|var| var.1.clone())
    }
    fn get_global_var(&self, name: &str) -> Option<String> { todo!() }

    fn lookup_field(&self, reference: FieldReference) -> Option<String> {
        todo!()
    }
}
