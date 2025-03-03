
use super::database_mgr::DatabaseMgr;
use crate::dbobjects::reference::{FieldReference, TableOccurrenceReference};
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

    fn lookup_field(&self, reference: FieldReference) -> Result<Option<String>, String> {
        let cur_window = self.window_mgr.windows.get(&self.state.active_window).unwrap();
        let cur_layout_id = cur_window.layout_id;
        let db = self.database_mgr.databases.get(&self.state.active_database).unwrap();
        let cur_occurrence_ref = db.file.layouts.get(cur_layout_id as usize).unwrap().occurrence.clone();

        let cur_set = cur_window.found_sets.iter()
            .find(|set| set.table_occurrence_ref == TableOccurrenceReference { 
                data_source: 0, 
                table_occurrence_id: cur_occurrence_ref.table_occurrence_id
            }).unwrap();

        if cur_set.cursor.is_none() {
            return Ok(None)
        }
        let record_id = cur_set.records[cur_set.cursor.unwrap() as usize];
        return Ok(self.database_mgr.get_field((cur_occurrence_ref, record_id), reference, &self.state.active_database))
    }
}
