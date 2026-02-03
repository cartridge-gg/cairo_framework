use crate::TableStructure;

pub trait RecordPrimary<impl Table: TableStructure, Record> {
    fn record_primary(self: @Table::Record) -> @Table::Primary;
}

pub trait PrimaryTrait<T> {
    fn to_felt252(self: @T) -> felt252;
}

impl PrimaryTraitImpl<T, +Copy<T>, +Into<T, felt252>> of PrimaryTrait<T> {
    fn to_felt252(self: @T) -> felt252 {
        (*self).into()
    }
}
