use syn::{Field, FnArg, ItemEnum, ItemFn, ItemStruct, Type, Variant};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsStruct {
    pub name: String,
    pub fields: Vec<RsField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsField {
    pub name: String,
    pub ty: RsType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RsType {
    Struct(RsStruct),
    Enum(RsEnum),
    Tuple(Vec<RsType>),
    Fn(RsFn),
    Primitive(RsPrimitive),
    Pointer(Box<RsType>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsEnum {
    pub name: String,
    pub variants: Vec<RsVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsVariant {
    pub name: String,
    pub fields: Vec<RsField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsFn {
    pub name: String,
    pub args: Vec<RsField>,
    pub ret: Box<RsType>,
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

impl From<ItemFn> for RsFn {
    fn from(value: ItemFn) -> Self {
        let name = value.sig.ident.to_string();
        let args = value.sig.inputs.into_iter().map(|arg| arg.into()).collect();
        let ret = match value.sig.output {
            syn::ReturnType::Default => RsType::Primitive(RsPrimitive::Unit),
            syn::ReturnType::Type(_, ty) => RsType::from(*ty),
        };
        Self {
            name,
            args,
            ret: Box::new(ret),
        }
    }
}

impl From<FnArg> for RsField {
    fn from(value: FnArg) -> Self {
        match value {
            syn::FnArg::Typed(ty) => {
                let name = match *ty.pat {
                    syn::Pat::Ident(ref ident) => ident.ident.to_string(),
                    _ => panic!("Function arguments must be named"),
                };
                let ty = RsType::from(*ty.ty);
                Self { name, ty }
            }
            _ => panic!("Function arguments must be named"),
        }
    }
}

impl From<Field> for RsField {
    fn from(value: Field) -> Self {
        let name = match value.ident {
            Some(ident) => ident.to_string(),
            None => panic!("Struct fields must be named"),
        };
        let ty = RsType::from(value.ty);
        Self { name, ty }
    }
}

impl From<ItemStruct> for RsStruct {
    fn from(value: ItemStruct) -> Self {
        let name = value.ident.to_string();
        let fields =
            value.fields.into_iter().map(|field| field.into()).collect();
        Self { name, fields }
    }
}

impl From<ItemEnum> for RsEnum {
    fn from(value: ItemEnum) -> Self {
        let name = value.ident.to_string();
        let variants = value
            .variants
            .into_iter()
            .map(|variant| variant.into())
            .collect();
        Self { name, variants }
    }
}

impl From<Variant> for RsVariant {
    fn from(value: Variant) -> Self {
        let name = value.ident.to_string();
        let fields = match value.fields {
            syn::Fields::Named(fields) => {
                fields.named.into_iter().map(|field| field.into()).collect()
            }
            syn::Fields::Unnamed(fields) => fields
                .unnamed
                .into_iter()
                .map(|field| field.into())
                .collect(),
            syn::Fields::Unit => vec![],
        };
        Self { name, fields }
    }
}

impl From<Type> for RsType {
    fn from(value: Type) -> Self {
        match value {
            Type::Path(path) => {
                let path = path.path;
                let segments = path.segments;
                let mut segments = segments.into_iter();
                let first = segments.next().unwrap();
                let name = first.ident.to_string();
                let ty = match name.as_str() {
                    "i8" => RsPrimitive::I8,
                    "i16" => RsPrimitive::I16,
                    "i32" => RsPrimitive::I32,
                    "i64" => RsPrimitive::I64,
                    "i128" => RsPrimitive::I128,
                    "u8" => RsPrimitive::U8,
                    "u16" => RsPrimitive::U16,
                    "u32" => RsPrimitive::U32,
                    "u64" => RsPrimitive::U64,
                    "u128" => RsPrimitive::U128,
                    "f32" => RsPrimitive::F32,
                    "f64" => RsPrimitive::F64,
                    "bool" => RsPrimitive::Bool,
                    "char" => RsPrimitive::Char,
                    "str" => RsPrimitive::Str,
                    "()" => RsPrimitive::Unit,
                    "String" => RsPrimitive::String,
                    _ => {
                        panic!("Unsupported type: {}", name);
                    }
                };
                let mut res = RsType::Primitive(ty);
                for segment in segments {
                    let name = segment.ident.to_string();
                    let ty_ = match name.as_str() {
                        "Option" => {
                            let ty = segment.arguments;
                            let ty = match ty {
                                syn::PathArguments::AngleBracketed(
                                    syn::AngleBracketedGenericArguments {
                                        args,
                                        ..
                                    },
                                ) => {
                                    let mut args = args.into_iter();
                                    let arg = args.next().unwrap();
                                    match arg {
                                        syn::GenericArgument::Type(ty) => ty,
                                        _ => panic!("Unsupported type"),
                                    }
                                }
                                _ => panic!("Unsupported type"),
                            };
                            RsType::Pointer(Box::new(RsType::from(ty)))
                        }
                        _ => panic!("Unsupported type"),
                    };
                    res = ty_;
                }
                res
            }
            Type::Tuple(tuple) => {
                let elems = tuple.elems;
                let elems = elems.into_iter();
                let elems = elems.map(|elem| RsType::from(elem));
                let elems = elems.collect();
                RsType::Tuple(elems)
            }
            Type::Ptr(ptr) => {
                let ty = *ptr.elem;
                let ty = RsType::from(ty);
                RsType::Pointer(Box::new(ty))
            }
            _ => panic!("Unsupported type"),
        }
    }
}
