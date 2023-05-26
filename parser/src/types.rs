use std::{error::Error, fmt::Display};

use syn::{
    Expr, Field, FnArg, ItemEnum, ItemFn, ItemStruct, Lit, Type, TypeArray,
    TypePath, Variant,
};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RsType {
    Struct(RsStruct),
    Enum(RsEnum),
    Primitive(RsPrimitive),
    Tuple(RsTuple),
    Array(RsArray),
    Slice(RsSlice),
    Func(RsFn),
    Pointer(RsPointer),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsStruct {
    pub name: String,
    pub fields: Vec<RsField>,
}

impl RsStruct {
    pub fn new(name: String, fields: Vec<RsField>) -> Self {
        Self { name, fields }
    }
}

impl From<RsStruct> for RsType {
    fn from(s: RsStruct) -> Self {
        Self::Struct(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsField {
    pub name: String,
    pub ty: RsType,
}

impl RsField {
    pub fn new(name: String, ty: RsType) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsEnum {
    pub name: String,
    pub variants: Vec<RsVariant>,
}

impl RsEnum {
    pub fn new(name: String, variants: Vec<RsVariant>) -> Self {
        Self { name, variants }
    }
}

impl From<RsEnum> for RsType {
    fn from(e: RsEnum) -> Self {
        Self::Enum(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsVariant {
    pub name: String,
    pub fields: Vec<RsField>,
}

impl RsVariant {
    pub fn new(name: String, fields: Vec<RsField>) -> Self {
        Self { name, fields }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsFn {
    pub name: String,
    pub args: Vec<RsField>,
    pub ret: Box<RsType>,
}

impl RsFn {
    pub fn new(name: String, args: Vec<RsField>, ret: RsType) -> Self {
        Self {
            name,
            args,
            ret: Box::new(ret),
        }
    }
}

impl From<RsFn> for RsType {
    fn from(f: RsFn) -> Self {
        Self::Func(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsArray {
    pub ty: Box<RsType>,
    pub len: usize,
}

impl RsArray {
    pub fn new(ty: RsType, len: usize) -> Self {
        Self {
            ty: Box::new(ty),
            len,
        }
    }
}

impl From<RsArray> for RsType {
    fn from(a: RsArray) -> Self {
        Self::Array(a)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RsPrimitive {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Char,
    Str,
    String,
    Unit,
}

impl From<RsPrimitive> for RsType {
    fn from(p: RsPrimitive) -> Self {
        Self::Primitive(p)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsPointer {
    pub ty: Box<RsType>,
    pub mutable: bool,
}

impl RsPointer {
    pub fn new(ty: RsType, mutable: bool) -> Self {
        Self {
            ty: Box::new(ty),
            mutable,
        }
    }
}

impl From<RsPointer> for RsType {
    fn from(p: RsPointer) -> Self {
        Self::Pointer(p)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsTuple {
    pub types: Vec<RsType>,
}

impl RsTuple {
    pub fn new(types: Vec<RsType>) -> Self {
        Self { types }
    }
}

impl From<RsTuple> for RsType {
    fn from(t: RsTuple) -> Self {
        Self::Tuple(t)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsSlice {
    pub ty: Box<RsType>,
}

impl RsSlice {
    pub fn new(ty: RsType) -> Self {
        Self { ty: Box::new(ty) }
    }
}

impl From<RsSlice> for RsType {
    fn from(s: RsSlice) -> Self {
        Self::Slice(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConversionError {
    UnnamedField { ty: String, msg: String },
    UnknownPrimitive { ty: String, msg: String },
    ReceiverField { ty: String, msg: String },
    UnknownType { ty: String, msg: String },
    GenericType { ty: String, msg: String },
    UnknownArrayLength { ty: String, msg: String },
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::UnnamedField { ty, msg } => {
                write!(f, "Invalid type: {} ({})", ty, msg)
            }
            ConversionError::UnknownPrimitive { ty, msg } => {
                write!(f, "Unsupported type: {} ({})", ty, msg)
            }
            ConversionError::ReceiverField { ty, msg } => {
                write!(f, "Receiver field: {} ({})", ty, msg)
            }
            ConversionError::UnknownType { ty, msg } => {
                write!(f, "Unknown Type: {} ({})", ty, msg)
            }
            ConversionError::GenericType { ty, msg } => {
                write!(f, "Unsupported Generics: {} ({})", ty, msg)
            }
            ConversionError::UnknownArrayLength { ty, msg } => {
                write!(f, "Unknown Array Length: {} ({})", ty, msg)
            }
        }
    }
}

impl Error for ConversionError {}

unsafe impl Send for ConversionError {}

unsafe impl Sync for ConversionError {}

impl TryFrom<&ItemFn> for RsFn {
    type Error = ConversionError;

    fn try_from(value: &ItemFn) -> Result<Self, Self::Error> {
        let name = value.sig.ident.to_string();
        let args = value
            .sig
            .inputs
            .into_iter()
            .map(|ref arg| RsField::try_from(arg))
            .collect::<Result<Vec<_>, _>>()?;
        let ret = match value.sig.output {
            syn::ReturnType::Default => RsType::Primitive(RsPrimitive::Unit),
            syn::ReturnType::Type(_, ty) => RsType::try_from(ty.as_ref())?,
        };
        Ok(Self::new(name, args, ret))
    }
}

impl TryFrom<&FnArg> for RsField {
    type Error = ConversionError;

    fn try_from(value: &FnArg) -> Result<Self, Self::Error> {
        match value {
            FnArg::Typed(arg) => {
                let name = match *arg.pat {
                    syn::Pat::Ident(ref ident) => ident.ident.to_string(),
                    _ => {
                        // TODO: better log info
                        log::error!("Got unnamed function argument: {:?}", arg);
                        return Err(ConversionError::UnnamedField {
                            ty: "FnArg::Typed".to_string(),
                            msg: "Unnamed fields are not supported".to_string(),
                        });
                    }
                };
                let ty = RsType::try_from(arg.ty.as_ref())?;
                Ok(Self { name, ty })
            }
            FnArg::Receiver(value) => {
                // TODO: better log info
                log::error!("Got function receiver: {:?}", value);
                Err(ConversionError::ReceiverField {
                    ty: "FnArg::Receiver".to_string(),
                    msg: "Function receivers are not supported".to_string(),
                })
            }
        }
    }
}

impl TryFrom<&Field> for RsField {
    type Error = ConversionError;

    fn try_from(value: &Field) -> Result<Self, Self::Error> {
        let name = match value.ident {
            Some(ident) => ident.to_string(),
            None => {
                log::error!("Got unnamed field: {:?}", value);
                return Err(ConversionError::UnnamedField {
                    ty: "Field".to_string(),
                    msg: "Unnamed fields are not supported".to_string(),
                });
            }
        };
        let ty = RsType::try_from(&value.ty)?;
        Ok(Self { name, ty })
    }
}

impl TryFrom<&ItemStruct> for RsStruct {
    type Error = ConversionError;

    fn try_from(value: &ItemStruct) -> Result<Self, Self::Error> {
        let name = value.ident.to_string();
        let fields = match value.fields {
            syn::Fields::Named(fields) => fields
                .named
                .into_iter()
                .map(|ref field| field.try_into())
                .collect::<Result<Vec<_>, _>>(),
            syn::Fields::Unnamed(fields) => fields
                .unnamed
                .into_iter()
                .map(|ref field| field.try_into())
                .collect::<Result<Vec<_>, _>>(),
            syn::Fields::Unit => Ok(Vec::new()),
        }?;
        Ok(Self { name, fields })
    }
}

impl TryFrom<&ItemEnum> for RsEnum {
    type Error = ConversionError;

    fn try_from(value: &ItemEnum) -> Result<Self, Self::Error> {
        let name = value.ident.to_string();
        let variants = value
            .variants
            .into_iter()
            .map(|ref variant| variant.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { name, variants })
    }
}

impl TryFrom<&Variant> for RsVariant {
    type Error = ConversionError;

    fn try_from(value: &Variant) -> Result<Self, Self::Error> {
        let name = value.ident.to_string();
        let fields = match value.fields {
            syn::Fields::Named(fields) => fields
                .named
                .into_iter()
                .map(|ref field| field.try_into())
                .collect::<Result<Vec<_>, _>>(),
            syn::Fields::Unnamed(fields) => fields
                .unnamed
                .into_iter()
                .map(|ref field| field.try_into())
                .collect::<Result<Vec<_>, _>>(),
            syn::Fields::Unit => Ok(Vec::new()),
        }?;
        Ok(Self { name, fields })
    }
}

impl TryFrom<&str> for RsPrimitive {
    type Error = ConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "i8" => Ok(Self::I8),
            "i16" => Ok(Self::I16),
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            "i128" => Ok(Self::I128),
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            "u128" => Ok(Self::U128),
            "f32" => Ok(Self::F32),
            "f64" => Ok(Self::F64),
            "bool" => Ok(Self::Bool),
            "Char" | "char" => Ok(Self::Char),
            "str" => Ok(Self::Str),
            "string" | "String" => Ok(Self::String),
            "unit" | "Unit" => Ok(Self::Unit),
            _ => {
                log::error!("Unknown primitive type: {}", value);
                Err(ConversionError::UnknownPrimitive {
                    ty: value.to_string(),
                    msg: "Unknown primitive type".to_string(),
                })
            }
        }
    }
}

impl TryFrom<&TypePath> for RsType {
    type Error = ConversionError;

    fn try_from(value: &TypePath) -> Result<Self, Self::Error> {
        let path = value.path;
        let segments = path.segments;
        let mut error = None;
        for segment in segments.iter() {
            let res: Result<RsType, ConversionError> = match segment.arguments {
                syn::PathArguments::None => {
                    let ident = segment.ident.to_string();
                    let primitive = RsPrimitive::try_from(ident.as_str())?;
                    Ok(Self::Primitive(primitive))
                }
                _ => {
                    log::error!(
                        "Generic types are not supported: {:?}",
                        segment
                    );
                    return Err(ConversionError::GenericType {
                        ty: "Type::Path".to_string(),
                        msg: "Generic types are not supported".to_string(),
                    });
                }
            };
            match res {
                Ok(ty) => return Ok(ty),
                Err(e) => error = Some(e),
            }
        }
        match error {
            Some(e) => Err(e),
            None => {
                log::error!("Unknown type: {:?}", value);
                Err(ConversionError::UnknownType {
                    ty: "Type::Path".to_string(),
                    msg: "Unknown type".to_string(),
                })
            }
        }
    }
}

impl TryFrom<&TypeArray> for RsArray {
    type Error = ConversionError;

    fn try_from(value: &TypeArray) -> Result<Self, Self::Error> {
        let ty = RsType::try_from(value.elem.as_ref())?;
        let len = value.len;
        // try convert len to usize
        // TODO: support non-literal array length
        let len = match len {
            Expr::Lit(lit) => match lit.lit {
                Lit::Int(int) => int.base10_parse::<usize>().unwrap(),
                _ => {
                    log::error!("Unknown array length: {:?}", len);
                    return Err(ConversionError::UnknownArrayLength {
                        ty: "Type::Array".to_string(),
                        msg: "Unknown array length".to_string(),
                    });
                }
            },
            _ => {
                log::error!("Unknown array length: {:?}", len);
                return Err(ConversionError::UnknownArrayLength {
                    ty: "Type::Array".to_string(),
                    msg: "Unknown array length".to_string(),
                });
            }
        };
        Ok(Self::new(ty, len))
    }
}

impl TryFrom<&TypeArray> for RsType {
    type Error = ConversionError;

    fn try_from(value: &TypeArray) -> Result<Self, Self::Error> {
        let arr = RsArray::try_from(value)?;
        Ok(Self::Array(arr))
    }
}

impl TryFrom<&Type> for RsType {
    type Error = ConversionError;

    fn try_from(value: &Type) -> Result<Self, Self::Error> {
        match value {
            Type::Path(ref path) => path.try_into(),
            Type::Array(ref arr) => arr.try_into(),
            Type::BareFn(ref func) => todo!(),
            Type::Group(ref group) => todo!(),
            Type::ImplTrait(ref impl_) => todo!(),
            Type::Infer(ref infer) => todo!(),
            Type::Macro(ref macro_) => todo!(),
            Type::Never(ref never) => todo!(),
            Type::Paren(ref paren) => todo!(),
            Type::Ptr(ref ptr) => todo!(),
            Type::Reference(ref reference) => todo!(),
            Type::Slice(ref slice) => todo!(),
            Type::TraitObject(ref trait_) => todo!(),
            Type::Tuple(ref tuple) => todo!(),
            Type::Verbatim(ref verbatim) => todo!(),
            _ => todo!(),
        }
    }
}
