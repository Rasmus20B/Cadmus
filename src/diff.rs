use std::collections::HashMap;

use crate::schema::{DBObjectStatus, Schema};

pub type DBObjectID = usize;
pub type DiffCollection = HashMap<DBObjectID, DBObjectStatus>;

pub fn get_table_diff(original: &Schema, updated: &Schema) -> DiffCollection {
    let mut result = DiffCollection::new();

    for u_table in &updated.tables {
        if let Some(matching) = original.tables.iter().find(|obj| obj.id == u_table.id) {
            if matching == u_table {
                result.insert(matching.id, DBObjectStatus::Unmodified);
            } else {
                result.insert(matching.id, DBObjectStatus::Modified);
            }
        } else {
            result.insert(u_table.id, DBObjectStatus::Created);
        }
    }

    for o_object in &original.tables {
        if updated.tables.iter().find(|obj| obj.id == o_object.id).is_none() {
            result.insert(o_object.id, DBObjectStatus::Deleted);
        }
    }
    result
}

// #[cfg(test)]
// mod tests {
//     use crate::{diff::DiffCollection, schema::{DBObject, DBObjectKind, TrackedSchema}};
//
//     use super::get_diff;
//
//     #[test]
//     fn diff_test() {
//         let mut o = TrackedSchema::new();
//         o.objects.extend(vec![
//             DBObject { id: 1, kind: DBObjectKind::Table, name: "Person".to_string() },
//             DBObject { id: 2, kind: DBObjectKind::Table, name: "Job".to_string() },
//             DBObject { id: 5, kind: DBObjectKind::Relation, name: "PersonToJob".to_string() },
//             DBObject { id: 1, kind: DBObjectKind::Field, name: "first_name".to_string() },
//             DBObject { id: 3, kind: DBObjectKind::ValueList, name: "names".to_string() },
//         ]);
//         let mut u = TrackedSchema::new();
//         u.objects.extend(vec![
//             DBObject { id: 1, kind: DBObjectKind::Table, name: "Employee".to_string() },
//             DBObject { id: 2, kind: DBObjectKind::Table, name: "Job".to_string() },
//             DBObject { id: 5, kind: DBObjectKind::Relation, name: "PersonToJob".to_string() },
//             DBObject { id: 1, kind: DBObjectKind::Field, name: "first_name".to_string() },
//             DBObject { id: 2, kind: DBObjectKind::Field, name: "last_name".to_string() },
//         ]);
//
//         let result = get_diff(&o, &u);
//
//         assert_eq!(result, DiffCollection {
//             created: vec![
//                 DBObject { id: 2, kind: DBObjectKind::Field, name: "last_name".to_string() },
//             ],
//             modified: vec![
//                 DBObject { id: 1, kind: DBObjectKind::Table, name: "Employee".to_string() },
//             ],
//             deleted: vec![
//                 DBObject { id: 3, kind: DBObjectKind::ValueList, name: "names".to_string() },
//             ],
//             unmodified: vec![
//                 DBObject { id: 2, kind: DBObjectKind::Table, name: "Job".to_string() },
//                 DBObject { id: 5, kind: DBObjectKind::Relation, name: "PersonToJob".to_string() },
//                 DBObject { id: 1, kind: DBObjectKind::Field, name: "first_name".to_string() },
//             ],
//         })
//
//     }
// }
//
//
//
//
//
//
//
