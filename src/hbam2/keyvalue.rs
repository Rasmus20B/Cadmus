
type Data = Vec<u8>;

enum B<'a> {
    KeyVals(Vec<&'a AB<'a>>),
    Collect(Vec<Data>),
    Single(Data),
}

struct AB<'a> {
    a: u8,
    b: B<'a>,
}

fn inspect(keyvalue: &AB) {
    match &keyvalue.b {
        B::KeyVals(inner) => {
            for keyval in inner {
                inspect(keyval)
            }
        },
        B::Collect(inner) => {
            println!("key: {}", keyvalue.a);
            for value in inner {
                println!("value: {:?}", value);
            }
        }
        B::Single(value) => {
            println!("key: {}, value: {:?}", keyvalue.a, value);
        }
    }
}

