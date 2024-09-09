use std::collections::BTreeMap;
use crate::fm_io::data_location::DataLocation;



pub struct DataCache {
    storage: BTreeMap<DataLocation, Vec<u8>>,
}
