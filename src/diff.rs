use crate::schema::{DBObject, Schema};

#[derive(Debug, PartialEq, Eq)]
pub struct DiffCollection {
    pub created: Vec<DBObject>,
    pub modified: Vec<DBObject>,
    pub deleted: Vec<DBObject>,
    pub unmodified: Vec<DBObject>,
}
impl DiffCollection {
    pub fn new() -> Self {
        Self {
            created: vec![],
            modified: vec![],
            deleted: vec![],
            unmodified: vec![],
        }
    }
}

pub fn get_diff(original: &Schema, updated: &Schema) -> DiffCollection {
    let mut result = DiffCollection::new();

    for u_object in &updated.objects {
        if let Some(matching) = original.objects.iter().find(|obj| obj.id == u_object.id && obj.kind == u_object.kind) {
            if matching == u_object {
                result.unmodified.push(u_object.clone());
            } else {
                result.modified.push(u_object.clone());
            }
        } else {
            result.created.push(u_object.clone())
        }
    }

    for o_object in &original.objects {
        if updated.objects.iter().find(|obj| obj.id == o_object.id && obj.kind == o_object.kind).is_none() {
            result.deleted.push(o_object.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::{diff::DiffCollection, schema::{DBObject, DBObjectKind, Schema}};

    use super::get_diff;

    #[test]
    fn diff_test() {
        let mut o = Schema::new();
        o.objects.extend(vec![
            DBObject { id: 1, kind: DBObjectKind::Table, name: "Person".to_string() },
            DBObject { id: 2, kind: DBObjectKind::Table, name: "Job".to_string() },
            DBObject { id: 5, kind: DBObjectKind::Relation, name: "PersonToJob".to_string() },
            DBObject { id: 1, kind: DBObjectKind::Field, name: "first_name".to_string() },
            DBObject { id: 3, kind: DBObjectKind::ValueList, name: "names".to_string() },
        ]);
        let mut u = Schema::new();
        u.objects.extend(vec![
            DBObject { id: 1, kind: DBObjectKind::Table, name: "Employee".to_string() },
            DBObject { id: 2, kind: DBObjectKind::Table, name: "Job".to_string() },
            DBObject { id: 5, kind: DBObjectKind::Relation, name: "PersonToJob".to_string() },
            DBObject { id: 1, kind: DBObjectKind::Field, name: "first_name".to_string() },
            DBObject { id: 2, kind: DBObjectKind::Field, name: "last_name".to_string() },
        ]);

        let result = get_diff(&o, &u);

        assert_eq!(result, DiffCollection {
            created: vec![
                DBObject { id: 2, kind: DBObjectKind::Field, name: "last_name".to_string() },
            ],
            modified: vec![
                DBObject { id: 1, kind: DBObjectKind::Table, name: "Employee".to_string() },
            ],
            deleted: vec![
                DBObject { id: 3, kind: DBObjectKind::ValueList, name: "names".to_string() },
            ],
            unmodified: vec![
                DBObject { id: 2, kind: DBObjectKind::Table, name: "Job".to_string() },
                DBObject { id: 5, kind: DBObjectKind::Relation, name: "PersonToJob".to_string() },
                DBObject { id: 1, kind: DBObjectKind::Field, name: "first_name".to_string() },
            ],
        })

    }
}







