use std::collections::HashSet;
use std::sync::OnceLock;

fn java_primitive_types() -> &'static HashSet<&'static str> {
    static HASHSET: OnceLock<HashSet<&str>> = OnceLock::new();
    HASHSET.get_or_init(|| {
        let mut set = HashSet::new();
        set.insert("boolean");
        set.insert("byte");
        set.insert("short");
        set.insert("int");
        set.insert("long");
        set.insert("float");
        set.insert("double");
        set.insert("char");
        set
    })
}

pub fn is_primitive_type(identifier: &str) -> bool {
    return java_primitive_types().contains(identifier);
}