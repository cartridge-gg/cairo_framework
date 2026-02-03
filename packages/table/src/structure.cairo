use introspect_types::type_def::TypeDefInline;
use introspect_types::{ChildDefs, TypeDef};


pub trait TableStructure {
    type Primary;
    type Record;
    const ATTRIBUTES_COUNT: u32;
    const COLUMN_COUNT: u32;
    #[inline(always)]
    fn serialise_attributes(
        ref data: Array<felt252>,
    ) {
        serialise_data::<_, { [1, 2, 3] }>(ref data);
    }
    #[inline(always)]
    fn serialize_primary(ref data: Array<felt252>);
    #[inline(always)]
    fn serialise_columns(ref table_def: Array<felt252>, ref children: ChildDefs);
}

#[inline]
pub fn serialise_column<
    const ID: felt252, const SIZE: u32, const DATA: [felt252; SIZE], T, impl TD: TypeDef<T>,
>(
    ref type_def: Array<felt252>, ref children: ChildDefs,
) {
    type_def.append(ID);
    type_def.append_span(DATA.span());
    TD::serialize_with_children(ref type_def, ref children);
}

#[inline]
pub fn serialize_primary<
    const SIZE: u32, const DATA: [felt252; SIZE], T, impl TD: TypeDefInline<T>,
>(
    ref data: Array<felt252>,
) {
    data.append_span(DATA.span());
    TD::serialize(ref data);
}

#[inline(always)]
pub fn serialise_data<const SIZE: u32, const DATA: [felt252; SIZE]>(ref data: Array<felt252>) {
    data.append_span(DATA.span());
}

