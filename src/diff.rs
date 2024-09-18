use crate::schema::{DBObject, Schema};

pub struct DiffCollection {
    created: Vec<DBObject>,
    modified: Vec<DBObject>,
    deleted: Vec<DBObject>,
    unmodified: Vec<DBObject>,
}

pub fn get_diff(original: &Schema, updated: &Schema) -> DiffCollection {
    unimplemented!()
    // result.extend(input.iter()
    //     .filter(|in_table| !original.iter()
    //         .map(|table| table.id).collect::<Vec<_>>()
    //         .contains(&in_table.id))
    //     .map(|table| TableKind::Created(table.clone())));
    //
    // result.extend(input.iter()
    //     .filter(|table| original.iter().map(|table| table.id).collect::<Vec<_>>().contains(&table.id))
    //     .filter(|table| !original.iter().map(|table| table.name.clone()).collect::<Vec<_>>().contains(&table.name))
    //     .map(|table| TableKind::Modified(table.clone()))
    //     .collect::<Vec<_>>());
    //
    // result.extend(input.iter()
    //     .filter(|table| original.contains(table))
    //     .map(|table| TableKind::UnModified(table.clone())));
    //
    // result.extend(original.iter()
    //     .filter(|table| !input
    //         .iter()
    //         .map(|new_table| new_table.id).collect::<Vec<_>>()
    //         .contains(&table.id))
    //     .map(|table| TableKind::Deleted(table.clone())).collect::<Vec<_>>()
    //     .into_iter());
    //
    // result.sort_by(|table1, table2| Table::from(table1.clone()).id.cmp(&Table::from(table2.clone()).id));
    // result
}
