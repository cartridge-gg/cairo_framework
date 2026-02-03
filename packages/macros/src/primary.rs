use crate::{Column, TableError};
use cairo_syntax_parser::CairoWrite;
use introspect_macros::{traits::MetaDataTrait, IAttribute, IAttributesTrait, INameTrait};
use introspect_types::PrimaryTypeDef;
use std::fmt::{Result as FmtResult, Write};

#[derive(Clone, Debug)]
pub struct Primary {
    pub name: String,
    pub attributes: Vec<IAttribute>,
    pub ty: String,
    pub type_def: PrimaryTypeDefVariant,
}

impl IAttributesTrait for Primary {
    fn iattributes(&self) -> &[IAttribute] {
        &self.attributes
    }
}

impl INameTrait for Primary {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Primary {
    pub fn cwrite_primary_data<W: Write>(&self, buf: &mut W, i_path: &str) -> FmtResult {
        write!(buf, "{i_path}::serialize_primary::<_, {{[")?;
        self.cwrite_meta_data(buf)?;
        buf.write_str("]}, ")?;
        self.ty.cwrite(buf)?;
        buf.write_str(">(data);\n")
    }
}

#[derive(Clone, Debug)]
pub enum PrimaryTypeDefVariant {
    Default,
    TypeDef(PrimaryTypeDef),
    Fn(String),
}

// impl IExtract for Primary {
//     type SyntaxType = Member;
//     type Error = TableError;
//     fn iextract(member: &mut Member) -> TableResult<Primary> {
//         let (TypeModAndName { type_mod, name }, attributes) = member.extract_attributes()?;
//         Ok(Primary {
//             name: name.unwrap_or_else(|| member.name.clone()),
//             member: member.name.clone(),
//             attributes,
//             ty: member.ty.clone(),
//             type_def: type_mod.get_type_def(&member.ty)?.try_into()?, //TODO: support type_mod,
//         })
//     }
// }

impl TryFrom<Column> for Primary {
    type Error = TableError;
    fn try_from(column: Column) -> Result<Self, Self::Error> {
        Ok(Primary {
            name: column.name,
            attributes: column.attributes,
            ty: column.ty,
            type_def: column.type_def.try_into()?,
        })
    }
}
