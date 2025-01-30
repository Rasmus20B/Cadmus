
/* We perform lookup from Quotes
 * If the lookup comes from File "0" then we don't interact with the HyperVisor.
 * If the lookup is NOT 0, then we ask the hypervisor to carry out the lookup for us.
 */

use std::collections::HashMap;

use crate::schema::DBObjectReference;

use super::window::Window;

type AliasList = HashMap<u32, u32>;

pub struct HyperVisor {
    windows: Vec<Window>,
    aliases: Vec<AliasList>
}

impl HyperVisor {
    pub fn global_lookup(from: u32, to: DBObjectReference) -> Vec<u8> {
        unimplemented!()
    } 
}
