use crate::id::id_string_to_felt;
use crate::{TableError, TableResult};
use cairo_syntax_parser::{Attribute, Member};
use introspect_macros::attribute::ExtractAttributes;
use introspect_macros::extraction::IExtractWith;
use introspect_macros::traits::MetaDataTrait;
use introspect_macros::utils::string_to_keccak_hex;
use introspect_macros::{
    AttributeParser, AttributeVariant, IAttributesTrait, IFieldTrait, INameTrait, ITyTrait,
    TypeDefVariant, TypeMod, TypeModMemberTrait, TypeModTrait,
};
use introspect_macros::{IAttribute, IntrospectError};
use introspect_rust_macros::macro_attributes;
use std::fmt::{Result as FmtResult, Write};

#[derive(Debug, Clone)]
pub struct Column {
    pub id: String,
    pub key: bool,
    pub name: String,
    pub member: String,
    pub selector: String,
    pub attributes: Vec<IAttribute>,
    pub ty: String,
    pub type_def: TypeDefVariant,
    pub member_impl_name: String,
}

impl INameTrait for Column {
    fn name(&self) -> &str {
        &self.name
    }
}

impl IFieldTrait for Column {
    fn field(&self) -> &str {
        &self.member
    }
}

impl ITyTrait for Column {
    fn ty(&self) -> &str {
        &self.ty
    }
}

impl IAttributesTrait for Column {
    fn iattributes(&self) -> &[IAttribute] {
        &self.attributes
    }
}

pub enum ColumnName {
    Default,
    Custom(String),
}

#[derive(Default)]
#[macro_attributes]
pub struct ColumnAttributes {
    #[skip]
    type_mod: TypeMod,
    name: String,
    id: String,
}

impl ColumnAttributes {}

impl TypeModMemberTrait for ColumnAttributes {
    fn get_mut_type_mod(&mut self) -> &mut TypeMod {
        &mut self.type_mod
    }
}

impl AttributeParser<Member> for ColumnAttributes {
    type Error = TableError;
    fn parse_attribute(
        &mut self,
        _module: &mut Member,
        attribute: Attribute,
    ) -> TableResult<Vec<AttributeVariant>> {
        if let Some(r) = self.extract_type_mod_return_empty(&attribute) {
            return r.map_err(From::from);
        }
        match attribute.path_str() {
            "name" => self.set_name_return_empty(attribute.single_unnamed_arg()?),
            "id" => self.set_id_return_empty(id_string_to_felt(attribute.single_unnamed_arg()?)),
            "index" => AttributeVariant::lazy_empty_i_attribute("index".to_string()),
            _ => attribute.into(),
        }
    }
}

impl IExtractWith for Column {
    type SyntaxType = Member;
    type Error = TableError;
    type Context = String;
    fn iextract_with(member: &mut Member, struct_name: &String) -> TableResult<Column> {
        let (ColumnAttributes { name, id, type_mod }, attributes) = member.extract_attributes()?;
        let member_impl_name = format! {"{struct_name}{}", member.name};
        let selector = string_to_keccak_hex(&member.name);
        let ty = member.ty.to_string();
        Ok(Column {
            id: id.unwrap_or_else(|| selector.clone()),
            name: name.unwrap_or_else(|| member.name.quoted()),
            member: member.name.clone(),
            key: member.has_name_only_attribute("key"),
            ty: ty.clone(),
            attributes,
            type_def: type_mod.get_type_def(&ty)?,
            member_impl_name,
            selector,
        })
    }
}

impl Column {
    pub fn serialize_member_call<const SELF: bool, W: Write>(&self, buf: &mut W) -> FmtResult {
        let self_str = if SELF { "self." } else { "" };
        writeln!(
            buf,
            "{}::serialize_member({self_str}{}, ref data);",
            self.member_impl_name, self.member
        )
    }

    pub fn cwrite_column_mod<W: Write>(&self, buf: &mut W) -> FmtResult {
        writeln!(buf, "pub const {} = {};", self.member, self.id)
    }

    pub fn cwrite_column_def<W: Write>(&self, buf: &mut W, i_path: &str) -> FmtResult {
        write!(buf, "{i_path}::serialise_column::<{}, _, {{[", self.id)?;
        self.cwrite_meta_data(buf)?;
        buf.write_str("]}, ")?;
        self.ty.cwrite(buf)?;
        buf.write_str(">(ref table_def, ref children);\n")
    }

    pub fn cwrite_member_impl<W: Write>(
        &self,
        buf: &mut W,
        i_path: &str,
        struct_impl_name: &str,
    ) -> FmtResult {
        writeln!(
            buf,
            "pub impl {} = {i_path}::MemberImpl<{struct_impl_name}, {}, {}>;",
            self.member_impl_name, self.id, self.ty
        )
    }
}
