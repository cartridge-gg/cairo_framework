use introspect_types::{ChildDefs, PrimaryDef};

pub trait TableStructure {
    type Primary;
    type Record;
}
