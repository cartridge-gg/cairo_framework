use std::fmt::{Result as FmtResult, Write};

use crate::primary::Primary;
use crate::{Column, TableError, TableResult};
use cairo_syntax_parser::{CairoWriteSlice, Member};
use introspect_macros::extraction::IExtractablesContext;
use introspect_macros::IAttribute;
use introspect_macros::{IAttributesTrait, IExtract};
use itertools::Itertools;

#[derive(Clone, Debug)]
pub enum KeyType {
    Primary(Primary),
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
    pub columns_mod_name: String,
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
        let key = if keys_index == 1 && item.members[0].is_primary() {
            KeyType::Primary(columns.remove(0).try_into()?)
        } else {
            KeyType::Custom(keys_index)
        };
        Ok(TableStructure {
            name: item.name.clone(),
            key,
            attributes: vec![],
            columns,
            impl_name: struct_impl_name_tpl(&item.name),
            columns_mod_name: columns_mod_name_tpl(&item.name),
        })
    }
}

impl TableStructure {
    pub fn cwrite_structure_impl<W: Write>(&self, buf: &mut W, i_path: &str) -> FmtResult {
        let impl_name = &self.impl_name;

        let mut column_defs = Vec::new();
        let mut member_impls = Vec::new();
        let mut column_id_consts = Vec::new();
        let mut tys = Vec::new();
        let mut serialize_member_calls = Vec::new();

        for column in &self.columns {
            column_id_consts.push(column.id_const());
            tys.push(&column.ty);
            column_defs.push(column.as_element_def_with(i_path, &self.columns_mod_name));
            member_impls.push(column.member_impl(i_table_path, &self.impl_name));
            if !column.key {
                serialize_member_calls.push(column.serialize_member_call::<true>());
            }
        }
        let (primary, key_impls) = match &self.key {
            KeyType::Primary(p) => (
                p,
                record_primary_impl_tpl(i_table_path, &self.name, &self.impl_name, &p.member),
            ),
            KeyType::Custom(k) => {
                let key_impls = match *k {
                    0 => "".to_string(),
                    1 => self.get_single_key_impls(i_table_path),
                    _ => self.get_keyed_impls(i_table_path),
                };
                (&default_primary_def(), key_impls)
            }
        };
        writeln!(buf, "pub impl {impl_name} of {i_path}::TableStructure {{",)?;
        writeln!(buf, "type Primary = {};", primary.ty)?;
        self.attributes.cwrite_attribute_count(buf)?;
        writeln!(buf, "const COLUMN_COUNT = {};", self.columns.len())?;
        buf.write_str("fn serialise_attributes(ref data: Array<felt252>) {\n")?;
        write!(buf, "{i_path}::serialise_data::<_,")?;
        self.attributes
            .cwrite_csv_wrapped_str(buf, "{[", "]}>(ref data);\n")?;
        buf.write_str("fn serialize_primary(ref data: Array<felt252>) {\n")?;
        primary.cwrite_primary_data(buf, i_path)?;
        buf.write_str("}\n")?;
        buf.write_str(
            "fn serialise_columns(ref table_def: Array<felt252>, ref children: ChildDefs) {\n",
        )?;
    }

    pub fn get_keyed_impls(&self, i_table_path: &str) -> String {
        let keys = self.columns.iter().filter(|c| c.key).collect::<Vec<_>>();
        let key_types: Vec<_> = keys.iter().map(|c| c.ty.to_cairo()).collect();
        let snapped_key_types = key_types.iter().map(|k| format!("@{k}")).join(",");
        let serialize_calls = keys
            .iter()
            .map(|c| c.serialize_member_call::<false>())
            .join("\n");
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
