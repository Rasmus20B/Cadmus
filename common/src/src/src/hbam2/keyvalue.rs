
type Data = Vec<u8>;

enum Value<'a> {
    KeyVals(Vec<&'a KeyValue<'a>>),
    Collect(Vec<Data>),
    Single(Data),
}

struct KeyValue<'a> {
    a: u8,
    b: Value<'a>,
}

fn inspect(keyvalue: &KeyValue) {
    match &keyvalue.b {
        Value::KeyVals(inner) => {
            for keyval in inner {
                inspect(keyval)
            }
        },
        Value::Collect(inner) => {
            println!("key: {}", keyvalue.a);
            for value in inner {
                println!("value: {:?}", value);
            }
        }
        Value::Single(value) => {
            println!("key: {}, value: {:?}", keyvalue.a, value);
        }
    }
}

