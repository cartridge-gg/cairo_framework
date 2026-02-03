use crate::primary::Primary;
use crate::{Column, TableError, TableResult};
use cairo_syntax_parser::{CairoWriteSlice, Member, Struct};
use introspect_macros::extraction::IExtractablesContext;
use introspect_macros::IAttribute;
use introspect_macros::{IAttributesTrait, IExtract};
use itertools::Itertools;
use std::fmt::{Result as FmtResult, Write};

#[derive(Clone, Debug)]
pub enum KeyType {
    Primary(String),
    Custom(usize),
}

#[derive(Clone, Debug)]
pub struct TableStructure {
    pub name: String,
    pub key: KeyType,
    pub primary: Primary,
    pub columns: Vec<Column>,
    pub attributes: Vec<IAttribute>,
    pub impl_name: String,
    pub column_mod_name: String,
}

trait TableMemberTrait {
    fn is_primary(&self) -> bool;
}

impl TableMemberTrait for Member {
    fn is_primary(&self) -> bool {
        self.ty.is_primary_type()
    }
}

pub fn get_keys_index(columns: &[Column]) -> TableResult<usize> {
    let mut position = 0;
    for (i, column) in columns.iter().enumerate() {
        if column.key {
            match position == i {
                true => position += 1,
                false => return Err(TableError::KeysNotFirst),
            }
        }
    }
    Ok(position)
}

impl IExtract for TableStructure {
    type SyntaxType = Struct;
    type Error = TableError;
    fn iextract(item: &mut Self::SyntaxType) -> Result<TableStructure, Self::Error> {
        let mut columns = item.members.iextracts_with(&item.name)?;
        let keys_index = get_keys_index(&columns)?;
        let name = item.name.clone();
        let impl_name = format!("{name}Structure");
        let column_mod_name = format!("{name}Column");
        let key = if keys_index == 1 && item.members[0].is_primary() {
            KeyType::Primary(columns.remove(0).try_into()?)
        } else {
            KeyType::Custom(keys_index)
        };

        Ok(TableStructure {
            name,
            key,
            attributes: vec![],
            columns,
            impl_name,
            column_mod_name,
        })
    }
}

impl TableStructure {
    pub fn cwrite_column_mods<W: Write>(&self, buf: &mut W, i_path: &str) -> FmtResult {
        writeln!(buf, "pub mod {} {{", self.columns_mod_name)?;
        self.columns
            .iter()
            .try_for_each(|c| c.cwrite_column_mod(buf))?;
        buf.write_str("}\n")
    }
    pub fn cwrite_structure_impl<W: Write>(&self, buf: &mut W, i_path: &str) -> FmtResult {
        let impl_name = &self.impl_name;
        writeln!(buf, "pub impl {impl_name} of {i_path}::TableStructure {{",)?;
        writeln!(buf, "type Primary = {};", self.primary.ty)?;
        self.attributes.cwrite_attribute_count(buf)?;
        writeln!(buf, "const COLUMN_COUNT = {};", self.columns.len())?;
        buf.write_str("fn serialise_attributes(ref data: Array<felt252>) {\n")?;
        write!(buf, "{i_path}::serialise_data::<_,")?;
        self.attributes
            .cwrite_csv_wrapped_str(buf, "{[", "]}>(ref data);\n")?;
        buf.write_str("fn serialize_primary(ref data: Array<felt252>) {\n")?;
        self.primary.cwrite_primary_data(buf, i_path)?;
        buf.write_str("}\n")?;
        buf.write_str(
            "fn serialise_columns(ref table_def: Array<felt252>, ref children: ChildDefs) {\n",
        )?;
        self.columns
            .iter()
            .try_for_each(|c| c.cwrite_column_def(buf, i_path))?;
        buf.write_str("}\n}\n")
    }

    pub fn cwrite_member_impls<W: Write>(&self, buf: &mut W, i_path: &str) -> FmtResult {
        self.columns
            .iter()
            .try_for_each(|c| c.cwrite_member_impl(buf, i_path, &self.impl_name))
    }

    pub fn cwrite_id_impls<W: Write>(&self, buf: &mut W, i_path: &str) -> FmtResult {
        let name = &self.name;
        let impl_name = &self.impl_name;
        match self.key {
            KeyType::Primary(field) => {
                writeln!(
                    buf,
                    "pub impl {name}RecordPrimary of {i_path}::RecordPrimary<{impl_name}, {name}> {{"
                )?;
                writeln!(
                    buf,
                    "fn record_id(self: @{name}) -> @{impl_name}::Primary {{"
                )?;
                writeln!(buf, "self.{field})")?;
                buf.write_str("}\n}\n")
            }
            KeyType::Custom(size) => {}
        }
    }

    pub fn cwrite_values_impls<W: Write>(&self, buf: &mut W, i_path: &str) -> FmtResult {
        let name = &self.name;
        let impl_name = &self.impl_name;
        writeln!(
            buf,
            "pub impl {name}RecordValues of {i_path}::RecordValues<{impl_name}, {name}> {{"
        )?;
        writeln!(
            buf,
            "fn serialize_values(self: @{name}, ref data: Array<felt252>) {{"
        )?;
        self.columns
            .iter()
            .filter(|c| !c.key)
            .try_for_each(|c| c.serialize_member_call(buf))?;
        buf.write_str("}\n}\n")
    }

    pub fn get_keyed_impls(&self, i_table_path: &str) -> String {
        let keys = self.columns.iter().filter(|c| c.key).collect::<Vec<_>>();
        let key_types: Vec<_> = keys.iter().map(|c| c.ty.to_cairo()).collect();
        let snapped_key_types = key_types.iter().map(|k| format!("@{k}")).join(",");
        let serialize_calls = keys.iter().map(|c| c.serialize_member_call()).join("\n");
        let key_members = keys.iter().map(|c| &c.member).join(",");
        let self_key_members = keys.iter().map(|c| format!("self.{}", c.member)).join(",");
        keyed_impls_tpl(
            i_table_path,
            &self.name,
            &self.impl_name,
            &key_types.join(","),
            &snapped_key_types,
            &serialize_calls,
            &key_members,
            &self_key_members,
        )
    }

    pub fn get_single_key_impls(&self, i_table_path: &str) -> String {
        let key = self.columns.iter().find(|c| c.key).unwrap();
        single_key_impls_tpl(
            i_table_path,
            &self.name,
            &self.impl_name,
            &key.ty.to_cairo(),
            &key.member,
            &key.member_impl_name,
        )
    }
}
