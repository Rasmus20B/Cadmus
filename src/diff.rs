use std::collections::HashMap;

use crate::schema::{DBObjectStatus, Schema};

pub type DBObjectID = usize;
pub type DiffCollection = HashMap<DBObjectID, DBObjectStatus>;

pub struct SchemaDiff {
    pub tables: DiffCollection,
    pub fields: DiffCollection,
    pub table_occurrences: DiffCollection,
    pub relations: DiffCollection,
    pub value_lists: DiffCollection,
    pub scripts: DiffCollection,
    pub tests: DiffCollection,
}

impl SchemaDiff {
    pub fn new() -> Self {
        Self {
            tables: DiffCollection::new(),
            fields: DiffCollection::new(),
            table_occurrences: DiffCollection::new(),
            relations: DiffCollection::new(),
            value_lists: DiffCollection::new(),
            scripts: DiffCollection::new(),
            tests: DiffCollection::new(),
        }
    }
}

pub fn get_table_diffs(original: &Schema, updated: &Schema) -> DiffCollection {
    let mut result = DiffCollection::new();
    for u_table in &updated.tables {
        if let Some(matching) = original.tables.iter().find(|obj| obj.1.id == u_table.1.id) {
            if matching == u_table {
                result.insert(matching.1.id, DBObjectStatus::Unmodified);
            } else {
                result.insert(matching.1.id, DBObjectStatus::Modified);
            }
        } else {
            result.insert(u_table.1.id, DBObjectStatus::Created);
        }
    }

    for o_object in &original.tables {
        if !updated.tables.iter().any(|obj| obj.1.id == o_object.1.id) {
            result.insert(o_object.1.id, DBObjectStatus::Deleted);
        }
    }
    result
}

pub fn get_table_occurrence_diffs(original: &Schema, updated: &Schema) -> DiffCollection {
    let mut result = DiffCollection::new();
    for u_table in &updated.table_occurrences {
        if let Some(matching) = original.table_occurrences.iter().find(|obj| obj.1.id == u_table.1.id) {
            if matching == u_table {
                result.insert(matching.1.id, DBObjectStatus::Unmodified);
            } else {
                result.insert(matching.1.id, DBObjectStatus::Modified);
            }
        } else {
            result.insert(u_table.1.id, DBObjectStatus::Created);
        }
    }

    for o_object in &original.table_occurrences {
        if !updated.table_occurrences.iter().any(|obj| obj.1.id == o_object.1.id) {
            result.insert(o_object.1.id, DBObjectStatus::Deleted);
        }
    }
    result
}

pub fn get_relations_diffs(original: &Schema, updated: &Schema) -> DiffCollection {
    let mut result = DiffCollection::new();
    for u_table in &updated.table_occurrences {
        if let Some(matching) = original.table_occurrences.iter().find(|obj| obj.1.id == u_table.1.id) {
            if matching == u_table {
                result.insert(matching.1.id, DBObjectStatus::Unmodified);
            } else {
                result.insert(matching.1.id, DBObjectStatus::Modified);
            }
        } else {
            result.insert(u_table.1.id, DBObjectStatus::Created);
        }
    }

    for o_object in &original.table_occurrences {
        if !updated.table_occurrences.iter().any(|obj| obj.1.id == o_object.1.id) {
            result.insert(o_object.1.id, DBObjectStatus::Deleted);
        }
    }
    result
}

pub fn get_diffs(original: &Schema, updated: &Schema) -> SchemaDiff {
    let mut result = SchemaDiff::new();
    result.tables.extend(get_table_diffs(original, updated));
    result.table_occurrences.extend(get_table_occurrence_diffs(original, updated));
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
