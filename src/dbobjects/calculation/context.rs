
use crate::dbobjects::reference::FieldReference;

pub trait CalculationContext {
    fn get_record_id(&self) -> u32;
    fn get_account_name(&self) -> String;
    fn get_active_field_contents(&self) -> String;
    fn get_active_fieldname(&self) -> String;
    fn get_host_ip_addr(&self) -> String;

    fn get_var(&self, name: &str) -> Option<String>;
    fn get_global_var(&self, name: &str) -> Option<String>;

    fn lookup_field(&self, reference: FieldReference) -> Option<String>;
}

pub struct DummyContext {
}

impl DummyContext {
    pub fn new() -> Self {
        Self {
        }
    } 
}

impl CalculationContext for DummyContext {

    fn get_record_id(&self) -> u32 { todo!() }
    fn get_account_name(&self) -> String { todo!() }
    fn get_active_field_contents(&self) -> String { todo!() }
    fn get_active_fieldname(&self) -> String { todo!() }
    fn get_host_ip_addr(&self) -> String { todo!() }

    fn get_var(&self, name: &str) -> Option<String> { todo!() }
    fn get_global_var(&self, name: &str) -> Option<String> { todo!() }

    fn lookup_field(&self, reference: FieldReference) -> Option<String> { todo!() }
}
