use flusty_parse::rust::types::{
    RsEnum, RsField, RsFn, RsPrimitive, RsStruct, RsType, RsVariant,
};

use crate::dart::{
    fill_dart_field_template, fill_dart_struct_template,
    fill_dart_type_def_template,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DartConversionErr {
    UnsupportedType(RsType),
    UnsupportedStruct(RsStruct),
    UnsupportedEnum(RsEnum),
    UnsupportedVariant(RsVariant),
    UnsupportedFn(RsFn),
    UnsupportedPrimitive(RsPrimitive),
    UnsupportedField(RsField),
}

pub trait ToDartFfi {
    fn to_dart_ffi(&self) -> Result<String, DartConversionErr>;
}

pub trait ToDart {
    fn to_dart(&self) -> Result<String, DartConversionErr>;
}

pub trait ToDartAnnotation {
    fn to_dart_annotation(&self) -> Option<String>;
}

impl ToDartFfi for RsStruct {
    fn to_dart_ffi(&self) -> Result<String, DartConversionErr> {
        let mut fields = Vec::new();
        for field in &self.fields {
            let field_str = fill_dart_field_template(
                field.ty.to_dart_annotation().unwrap_or_default(),
                field.ty.to_dart_ffi()?,
                field.name.clone(),
            );
            fields.push(field_str);
        }
        let field_str = fields.join("\n");
        Ok(fill_dart_struct_template(&self.name, field_str))
    }
}

impl ToDartFfi for RsField {
    fn to_dart_ffi(&self) -> Result<String, DartConversionErr> {
        return Ok(format!("{} {}", self.name, self.ty.to_dart_ffi()?));
    }
}

impl ToDart for RsField {
    fn to_dart(&self) -> Result<String, DartConversionErr> {
        return Ok(format!("{} {}", self.name, self.ty.to_dart()?));
    }
}

impl ToDartFfi for RsType {
    fn to_dart_ffi(&self) -> Result<String, DartConversionErr> {
        match self {
            RsType::Struct(s) => s.to_dart_ffi(),
            RsType::Enum(e) => e.to_dart_ffi(),
            RsType::Tuple(t) => Err(DartConversionErr::UnsupportedType(
                RsType::Tuple(t.clone()),
            )),
            RsType::Fn(f) => f.to_dart_ffi(),
            RsType::Primitive(p) => p.to_dart_ffi(),
            RsType::Pointer(p) => {
                Ok(format!("ffi.Pointer<{}>", p.to_dart_ffi()?))
            }
        }
    }
}

impl ToDartAnnotation for RsType {
    fn to_dart_annotation(&self) -> Option<String> {
        match self {
            RsType::Struct(s) => todo!(),
            RsType::Enum(e) => todo!(),
            RsType::Tuple(t) => {
                if t.is_empty() {
                    None
                } else {
                    Some("ffi.Packed".to_string())
                }
            }
            RsType::Fn(f) => todo!(),
            RsType::Primitive(p) => p.to_dart_annotation(),
            RsType::Pointer(p) => p.to_dart_annotation(),
        }
    }
}

impl ToDart for RsType {
    fn to_dart(&self) -> Result<String, DartConversionErr> {
        match self {
            RsType::Struct(s) => todo!(),
            RsType::Enum(e) => todo!(),
            RsType::Tuple(t) => Err(DartConversionErr::UnsupportedType(
                RsType::Tuple(t.clone()),
            )),
            RsType::Fn(f) => f.to_dart(),
            RsType::Primitive(p) => p.to_dart(),
            RsType::Pointer(p) => p.to_dart(),
        }
    }
}

impl ToDartFfi for RsEnum {
    fn to_dart_ffi(&self) -> Result<String, DartConversionErr> {
        todo!()
    }
}

impl ToDartFfi for RsVariant {
    fn to_dart_ffi(&self) -> Result<String, DartConversionErr> {
        todo!()
    }
}

impl ToDart for RsFn {
    fn to_dart(&self) -> Result<String, DartConversionErr> {
        let args = self
            .args
            .iter()
            .map(|arg| arg.to_dart().unwrap_or_default())
            .collect::<Vec<String>>()
            .join(", ");
        let ret = self.ret.to_dart()?;
        let res = fill_dart_type_def_template(&self.name, args, ret);
        Ok(res)
    }
}

impl ToDartFfi for RsFn {
    fn to_dart_ffi(&self) -> Result<String, DartConversionErr> {
        let args = self
            .args
            .iter()
            .map(|arg| arg.to_dart_ffi().unwrap_or_default())
            .collect::<Vec<String>>()
            .join(", ");
        let ret = self.ret.to_dart_ffi()?;
        let res = fill_dart_type_def_template(&self.name, args, ret);
        Ok(res)
    }
}

impl ToDartFfi for RsPrimitive {
    fn to_dart_ffi(&self) -> Result<String, DartConversionErr> {
        match self {
            RsPrimitive::I8 => Ok("ffi.Int8".to_string()),
            RsPrimitive::I16 => Ok("ffi.Int16".to_string()),
            RsPrimitive::I32 => Ok("ffi.Int32".to_string()),
            RsPrimitive::I64 => Ok("ffi.Int64".to_string()),
            RsPrimitive::I128 => {
                Err(DartConversionErr::UnsupportedPrimitive(RsPrimitive::I128))
            }
            RsPrimitive::U8 => Ok("ffi.Uint8".to_string()),
            RsPrimitive::U16 => Ok("ffi.Uint16".to_string()),
            RsPrimitive::U32 => Ok("ffi.Uint32".to_string()),
            RsPrimitive::U64 => {
                Err(DartConversionErr::UnsupportedPrimitive(RsPrimitive::U64))
            }
            RsPrimitive::U128 => {
                Err(DartConversionErr::UnsupportedPrimitive(RsPrimitive::U128))
            }
            RsPrimitive::F32 => Ok("ffi.Float".to_string()),
            RsPrimitive::F64 => Ok("ffi.Double".to_string()),
            RsPrimitive::Bool => Ok("ffi.Bool".to_string()),
            RsPrimitive::Char => Ok("ffi.Char".to_string()),
            RsPrimitive::Str => Ok("ffi.Pointer<ffi.Utf8>".to_string()),
            RsPrimitive::String => Ok("ffi.String".to_string()),
            RsPrimitive::Unit => Ok("ffi.Void".to_string()),
        }
    }
}

impl ToDartAnnotation for RsPrimitive {
    fn to_dart_annotation(&self) -> Option<String> {
        match self {
            RsPrimitive::I8 => Some("@Int8()".to_string()),
            RsPrimitive::I16 => Some("@Int16()".to_string()),
            RsPrimitive::I32 => Some("@Int32()".to_string()),
            RsPrimitive::I64 => Some("@Int64()".to_string()),
            RsPrimitive::I128 => None,
            RsPrimitive::U8 => Some("@Uint8()".to_string()),
            RsPrimitive::U16 => Some("@Uint16()".to_string()),
            RsPrimitive::U32 => Some("@Uint32()".to_string()),
            RsPrimitive::U64 => None,
            RsPrimitive::U128 => None,
            RsPrimitive::F32 => Some("@Float()".to_string()),
            RsPrimitive::F64 => Some("@Double()".to_string()),
            RsPrimitive::Bool => Some("@Bool()".to_string()),
            RsPrimitive::Char => None,
            RsPrimitive::Str => None,
            RsPrimitive::String => None,
            RsPrimitive::Unit => None,
        }
    }
}

impl ToDart for RsPrimitive {
    fn to_dart(&self) -> Result<String, DartConversionErr> {
        match self {
            RsPrimitive::I8 => Ok("int".to_string()),
            RsPrimitive::I16 => Ok("int".to_string()),
            RsPrimitive::I32 => Ok("int".to_string()),
            RsPrimitive::I64 => Ok("int".to_string()),
            RsPrimitive::I128 => {
                Err(DartConversionErr::UnsupportedPrimitive(RsPrimitive::I128))
            }
            RsPrimitive::U8 => Ok("int".to_string()),
            RsPrimitive::U16 => Ok("int".to_string()),
            RsPrimitive::U32 => Ok("int".to_string()),
            RsPrimitive::U64 => {
                Err(DartConversionErr::UnsupportedPrimitive(RsPrimitive::U64))
            }
            RsPrimitive::U128 => {
                Err(DartConversionErr::UnsupportedPrimitive(RsPrimitive::U128))
            }
            RsPrimitive::F32 => Ok("double".to_string()),
            RsPrimitive::F64 => Ok("double".to_string()),
            RsPrimitive::Bool => Ok("bool".to_string()),
            RsPrimitive::Char => Ok("String".to_string()),
            RsPrimitive::Str => Ok("String".to_string()),
            RsPrimitive::String => Ok("String".to_string()),
            RsPrimitive::Unit => Ok("void".to_string()),
        }
    }
}
