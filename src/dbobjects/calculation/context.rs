
pub trait CalculationContext {
    fn get_record_id() -> u32;
    fn get_account_name() -> String;
    fn get_active_field_contents() -> String;
    fn get_active_fieldname() -> String;
    fn get_host_ip_addr() -> String;

}
