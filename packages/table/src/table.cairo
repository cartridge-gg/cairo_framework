use cgg_utils::{ToSnapshotOf, ToSpan};
use core::array;
use introspect_events::database::selectors::{CreateTable, InsertField};
use introspect_events::database::{
    CreateColumnSet, DeleteField, DeleteFields, DeleteRecord, DeleteRecords, DeletesField,
    DeletesFields, InsertsField,
};
use introspect_events::{EmitEvent, EmitRawEvent};
use introspect_types::ChildDefs;
use crate::field::RecordsField;
use crate::record::RecordIds;
use crate::recordable_events::{Emittable, EmittableBatch, EmittableFields, EmittableFieldsBatch};
use crate::{ColumnSet, Member, RecordId, TableStructure};

pub trait ITable {
    impl Table: TableStructure;
    const ID: felt252;
    const ATTRIBUTES_COUNT: u32;
    fn name() -> ByteArray;
    fn serialize_name(ref output: Array<felt252>);
    fn serialise_attributes(ref data: Array<felt252>);
    fn register_table(
        ref children: ChildDefs,
    ) {
        let mut data: Array<felt252> = Default::default();
        data.append(Self::ID);
        Self::serialize_name(ref data);
        data.append(Self::ATTRIBUTES_COUNT.into() + Self::Table::ATTRIBUTES_COUNT.into());
        Self::serialise_attributes(ref data);
        Self::Table::serialise_attributes(ref data);
        Self::Table::serialize_primary(ref data);
        Self::Table::serialise_columns(ref data, ref children);
        CreateTable.emit_event_data(data);
    }
    fn crate_column_set<
        Set, const SIZE: usize, impl ColumnSet: ColumnSet<Self::Table, Set, SIZE>,
    >() {
        CreateColumnSet { id: ColumnSet::GROUP_ID, columns: ColumnSet::column_ids() }.emit()
    }
    fn insert<Item, impl RE: Emittable<Self::ID, Self::Table, Item>, +Drop<Item>>(
        record: Item,
    ) {
        RE::emit_item(@record);
    }
    fn inserts<Items, impl RE: EmittableBatch<Self::ID, Self::Table, Items>, +Drop<Items>>(
        records: Items,
    ) {
        RE::emit_batch(records);
    }
    fn insert_field<
        const ID: felt252,
        ToId,
        ToField,
        impl RId: RecordId<Self::Table, ToId>,
        impl Member: Member<Self::Table, Self::Table::Record, ID>,
        impl SF: ToSnapshotOf<ToField, Member::Type>,
        +Drop<ToId>,
        +Drop<ToField>,
    >(
        id: ToId, field: ToField,
    ) {
        let mut data = array![Self::ID, RId::record_id(@id), ID];
        Member::serialize_member(SF::to_snapshot(field), ref data);
        InsertField.emit_event_data(data);
    }
    fn inserts_field<
        const ID: felt252,
        impl Member: Member<Self::Table, Self::Table::Record, ID>,
        Items,
        impl Field: RecordsField<Self::Table, ID, Member, Items>,
    >(
        items: Items,
    ) {
        let entries = Field::serialise_to_entries(items);
        InsertsField { table: Self::ID, column: ID, entries }.emit();
    }
    fn insert_fields<Item, impl RE: EmittableFields<Self::ID, Self::Table, Item>, +Drop<Item>>(
        record: Item,
    ) {
        RE::emit_fields(@record);
    }

    fn inserts_fields<
        Items, impl RE: EmittableFieldsBatch<Self::ID, Self::Table, Items>, +Drop<Items>,
    >(
        records: Items,
    ) {
        RE::emit_fields_batch(records);
    }
    fn delete_record<ToId, impl RID: RecordId<Self::Table, ToId>, +Drop<ToId>>(
        id: ToId,
    ) {
        DeleteRecord { table: Self::ID, row: RID::record_id(@id) }.emit();
    }
    fn delete_records<ToIds, impl Ids: RecordIds<Self::Table, ToIds>, +Drop<ToIds>>(
        ids: ToIds,
    ) {
        DeleteRecords { table: Self::ID, rows: Ids::record_ids(ids) }.emit();
    }
    fn delete_field<
        const COLUMN_ID: felt252,
        ToId,
        impl RID: RecordId<Self::Table, ToId>,
        impl Member: Member<Self::Table, Self::Table::Record, COLUMN_ID>,
        +Drop<ToId>,
    >(
        id: ToId,
    ) {
        DeleteField { table: Self::ID, row: RID::record_id(@id), column: COLUMN_ID }.emit();
    }
    fn deletes_field<
        const COLUMN_ID: felt252,
        ToIds,
        impl TID: RecordIds<Self::Table, ToIds>,
        impl Member: Member<Self::Table, Self::Table::Record, COLUMN_ID>,
    >(
        ids: ToIds,
    ) {
        DeletesField { table: Self::ID, rows: TID::record_ids(ids), column: COLUMN_ID }.emit();
    }
    fn delete_fields<
        ToId,
        ColumnIds,
        impl Id: RecordId<Self::Table, ToId>,
        +ToSpan<ColumnIds, felt252>,
        +Drop<ToId>,
        +Drop<ColumnIds>,
    >(
        id: ToId, columns: ColumnIds,
    ) {
        DeleteFields { table: Self::ID, row: Id::record_id(@id), columns: columns.to_span() }
            .emit();
    }
    fn deletes_fields<
        ToIds,
        ColumnIds,
        impl Ids: RecordIds<Self::Table, ToIds>,
        +ToSpan<ColumnIds, felt252>,
        +Drop<ColumnIds>,
    >(
        ids: ToIds, columns: ColumnIds,
    ) {
        DeletesFields { table: Self::ID, rows: Ids::record_ids(ids), columns: columns.to_span() }
            .emit();
    }
}
