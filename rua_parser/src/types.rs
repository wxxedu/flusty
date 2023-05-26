//! This module contains the data structures that represent Rust types.
#![warn(missing_docs)]
#![deny(clippy::all)]

use std::{
    error::Error,
    fmt::{Debug, Display},
};

use proc_macro2::Span;
use syn::{
    spanned::Spanned, Expr, Field, FnArg, ItemEnum, ItemFn, ItemStruct, Pat,
    ReturnType, Type, TypeArray, TypePath, TypePtr, TypeSlice, TypeTuple,
    Variant,
};

/// Represents something that can be described.
pub trait Descriptable {
    /// Returns the description of the object.
    fn description(&self) -> String;
}

impl Descriptable for &ItemStruct {
    fn description(&self) -> String {
        format!("struct {}", self.ident)
    }
}

impl Descriptable for &Field {
    fn description(&self) -> String {
        format!(
            "field {}: {}",
            self.ident.as_ref().unwrap(),
            (&self.ty).description()
        )
    }
}

impl Descriptable for &FnArg {
    fn description(&self) -> String {
        match self {
            FnArg::Receiver(_) => "self".to_string(),
            FnArg::Typed(pat) => match &*pat.pat {
                Pat::Ident(ident) => {
                    format!("{}: {}", ident.ident, (&*pat.ty).description())
                }
                _ => panic!("Unsupported pattern: {:?}", pat),
            },
        }
    }
}

impl Descriptable for &ItemEnum {
    fn description(&self) -> String {
        let variants_str = self
            .variants
            .iter()
            .map(|variant| variant.description())
            .collect::<Vec<_>>()
            .join(", ");
        format!("enum {} {{ {} }}", self.ident, variants_str)
    }
}

impl Descriptable for &Variant {
    fn description(&self) -> String {
        let fields_str = self
            .fields
            .iter()
            .map(|field| field.description())
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}({})", self.ident, fields_str)
    }
}

impl Descriptable for &ItemFn {
    fn description(&self) -> String {
        let args_str = self
            .sig
            .inputs
            .iter()
            .map(|arg| arg.description())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "fn {}({}) -> {}",
            self.sig.ident,
            args_str,
            (&self.sig.output).description()
        )
    }
}

impl Descriptable for &ReturnType {
    fn description(&self) -> String {
        match self {
            ReturnType::Default => "".to_string(),
            ReturnType::Type(_, ty) => {
                format!("-> {}", (ty.as_ref()).description())
            }
        }
    }
}

impl Descriptable for Expr {
    fn description(&self) -> String {
        todo!()
    }
}

impl Descriptable for &TypeArray {
    fn description(&self) -> String {
        format!(
            "[{}; {}]",
            (self.elem.as_ref()).description(),
            self.len.description()
        )
    }
}

impl Descriptable for &Type {
    fn description(&self) -> String {
        todo!()
    }
}

/// Represents a position in a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsPosition {
    /// The line.
    pub line: usize,
    /// The column.
    pub column: usize,
}

impl Display for RsPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}:{}", self.line, self.column)
    }
}

/// Represents a span in a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsSpan {
    /// The start position.
    pub start: RsPosition,
    /// The end position.
    pub end: RsPosition,
}

impl Display for RsSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

impl From<&Span> for RsSpan {
    fn from(value: &Span) -> Self {
        let start = value.start();
        let end = value.end();
        Self {
            start: RsPosition {
                line: start.line,
                column: start.column,
            },
            end: RsPosition {
                line: end.line,
                column: end.column,
            },
        }
    }
}

/// Represents a Rust type.
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConversionError {
    /// The source type.
    pub src: Option<String>,
    /// The destination type.
    pub dst: Option<String>,
    /// The message.
    pub msg: Option<String>,
    /// The data.
    pub data: Option<String>,
    /// The source error.
    pub source: Option<Box<ConversionError>>,
    /// The span.
    pub span: Option<RsSpan>,
}

/// A builder for `ConversionError`.
pub struct ConversionErrorBuilder {
    /// The error.
    error: ConversionError,
}

impl ConversionErrorBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            error: ConversionError::default(),
        }
    }

    /// Sets the source type.
    pub fn with_source_opt(&mut self, src: &Option<String>) -> &mut Self {
        self.error.src = src.clone();
        self
    }

    /// Sets the source type.
    pub fn with_source(&mut self, src: impl Into<String>) -> &mut Self {
        self.error.src = Some(src.into());
        self
    }

    /// Sets the destination type.
    pub fn with_destination_opt(&mut self, dst: &Option<String>) -> &mut Self {
        self.error.dst = dst.clone();
        self
    }

    /// Sets the destination type.
    pub fn with_destination(&mut self, dst: impl Into<String>) -> &mut Self {
        self.error.dst = Some(dst.into());
        self
    }

    /// Sets the message.
    pub fn with_message_opt(&mut self, msg: &Option<String>) -> &mut Self {
        self.error.msg = msg.clone();
        self
    }

    /// Sets the message.
    pub fn with_message(&mut self, msg: impl Into<String>) -> &mut Self {
        self.error.msg = Some(msg.into());
        self
    }

    /// Sets the data.
    pub fn with_data<T: Descriptable>(&mut self, data: &T) -> &mut Self {
        self.error.data = Some(data.description());
        self
    }

    /// Sets the source error.
    pub fn with_error_source(&mut self, source: ConversionError) -> &mut Self {
        self.error.source = Some(Box::new(source));
        self
    }

    /// Sets the span.
    pub fn with_span(&mut self, span: RsSpan) -> &mut Self {
        self.error.span = Some(span);
        self
    }

    /// Builds the error.
    pub fn build(&self) -> ConversionError {
        log::error!("{}", self.error);
        self.error.clone()
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let src = if let Some(src) = &self.src {
            format!(" from {}", src)
        } else {
            String::new()
        };
        let dst = if let Some(dst) = &self.dst {
            format!(" to {}", dst)
        } else {
            String::new()
        };
        let msg = if let Some(msg) = &self.msg {
            format!(": {}", msg)
        } else {
            String::new()
        };
        let data = if let Some(data) = &self.data {
            format!("\n\t- (data: {})", data)
        } else {
            String::new()
        };
        let span = if let Some(span) = &self.span {
            format!("\n\t- (span: {})", span)
        } else {
            String::new()
        };
        write!(
            f,
            "ConversionError: failed to convert{}{}{}{}{}",
            src, dst, msg, data, span
        )
    }
}

impl Error for ConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(source) = &self.source {
            Some(source.as_ref())
        } else {
            None
        }
    }
}

unsafe impl Send for ConversionError {}

unsafe impl Sync for ConversionError {}

/// The type of a module.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum RsModuleType {
    /// A module that is declared as a crate.
    #[default]
    CrateModule,
    /// A module that is declared as a submodule.
    SubModule {
        /// The name of the parent module.
        parent: String,
    },
}

impl Display for RsModuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RsModuleType::CrateModule => write!(f, "crate"),
            RsModuleType::SubModule { parent } => {
                write!(f, "submodule of {}", parent)
            }
        }
    }
}

/// The data structure of a module.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RsModule {
    /// The name of the module.
    pub name: String,
    /// The type of the module, see [RsModuleType].
    pub ty: RsModuleType,
    /// The submodules of the module.
    pub submodules: Vec<RsModule>,
    /// The structures of the module.
    pub structs: Vec<RsStruct>,
    /// The enums of the module.
    pub enums: Vec<RsEnum>,
    /// The functions of the module.
    pub funcs: Vec<RsFn>,
}

impl Display for RsModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} (#structs: {}, #enums: {}, $funcs: {})",
            self.ty,
            self.name,
            self.structs.len(),
            self.enums.len(),
            self.funcs.len()
        )
    }
}

impl RsModule {
    /// Creates a new module.
    pub fn new(
        name: String,
        ty: RsModuleType,
        submodules: Vec<RsModule>,
        types: Vec<RsType>,
    ) -> RsModule {
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut funcs = Vec::new();
        for ty in types {
            match ty {
                RsType::Struct(s) => structs.push(s),
                RsType::Enum(e) => enums.push(e),
                RsType::Func(f) => funcs.push(f),
                ty => {
                    log::warn!("Ignoring type {} at file level", ty);
                }
            }
        }
        RsModule {
            name,
            ty,
            submodules,
            structs,
            enums,
            funcs,
        }
    }
}

/// Represents a type in Rust.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RsType {
    /// Wraps around a [RsStruct].
    Struct(RsStruct),
    /// Wraps around a [RsEnum].
    Enum(RsEnum),
    /// Wraps around a [RsPrimitive].
    Primitive(RsPrimitive),
    /// Wraps around a [RsTuple].
    Tuple(RsTuple),
    /// Wraps around a [RsArray].
    Array(RsArray),
    /// Wraps around a [RsSlice].
    Slice(RsSlice),
    /// Wraps around a [RsFn].
    Func(RsFn),
    /// Wraps around a [RsPointer].
    Pointer(RsPointer),
    /// Wraps a unit type.
    Unit,
}

impl Display for RsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RsType::Struct(ref v) => write!(f, "type {}", v),
            RsType::Enum(ref v) => write!(f, "type {}", v),
            RsType::Primitive(ref v) => write!(f, "type {}", v),
            RsType::Tuple(ref v) => write!(f, "type {}", v),
            RsType::Array(ref v) => write!(f, "type {}", v),
            RsType::Slice(ref v) => write!(f, "type {}", v),
            RsType::Func(ref v) => write!(f, "type {}", v),
            RsType::Pointer(ref v) => write!(f, "type {}", v),
            RsType::Unit => write!(f, "type ()"),
        }
    }
}

impl TryFrom<&Type> for RsType {
    type Error = ConversionError;

    fn try_from(value: &Type) -> Result<Self, Self::Error> {
        todo!()
    }
}

/// Represents a struct in Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsStruct {
    /// The name of the struct.
    pub name: String,
    /// The fields of the struct.
    pub fields: Vec<RsField>,
}

impl Display for RsStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let field_str = self
            .fields
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "struct {} ({})", self.name, field_str)
    }
}

impl RsStruct {
    /// Creates a new struct.
    pub fn new(name: String, fields: Vec<RsField>) -> Self {
        Self { name, fields }
    }
}

impl From<RsStruct> for RsType {
    fn from(s: RsStruct) -> Self {
        Self::Struct(s)
    }
}

impl TryFrom<&ItemStruct> for RsStruct {
    type Error = ConversionError;

    fn try_from(value: &ItemStruct) -> Result<Self, Self::Error> {
        let name = value.ident.to_string();
        let fields = value
            .fields
            .iter()
            .map(RsField::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                ConversionErrorBuilder::new()
                    .with_source_opt(&e.src)
                    .with_destination("RsStruct")
                    .with_data(&value)
                    .with_error_source(e)
                    .with_span((&value.span()).into())
                    .build()
            })?;
        Ok(Self::new(name, fields))
    }
}

/// Represents a variant of an enum in Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsField {
    /// The name of the field.
    pub name: String,
    /// The type of the field.
    pub ty: RsType,
}

impl Display for RsField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl RsField {
    /// Creates a new field.
    pub fn new(name: String, ty: RsType) -> Self {
        Self { name, ty }
    }
}

impl TryFrom<&Field> for RsField {
    type Error = ConversionError;

    fn try_from(value: &Field) -> Result<Self, Self::Error> {
        let name =
            value.ident.as_ref().map(|i| i.to_string()).ok_or_else(|| {
                ConversionErrorBuilder::new()
                    .with_source("Field")
                    .with_destination("RsField")
                    .with_data(&value)
                    .with_message("Field must have an identifier")
                    .with_span((&value.span()).into())
                    .build()
            })?;
        let ty = RsType::try_from(&value.ty).map_err(|e| {
            ConversionErrorBuilder::new()
                .with_source_opt(&e.src)
                .with_destination("RsField")
                .with_data(&value)
                .with_error_source(e)
                .with_span((&value.span()).into())
                .build()
        })?;
        Ok(Self::new(name, ty))
    }
}

impl TryFrom<&FnArg> for RsField {
    type Error = ConversionError;

    fn try_from(value: &FnArg) -> Result<Self, Self::Error> {
        let (name, ty) = match value {
            FnArg::Typed(ty) => {
                let name = match ty.pat.as_ref() {
                    Pat::Ident(i) => i.ident.to_string(),
                    _ => {
                        return Err(ConversionErrorBuilder::new()
                            .with_source("FnArg")
                            .with_destination("RsField")
                            .with_data(&value)
                            .with_message(
                                "FnArg::Typed must have an identifier",
                            )
                            .with_span((&value.span()).into())
                            .build());
                    }
                };
                let ty = RsType::try_from(ty.ty.as_ref()).map_err(|e| {
                    ConversionErrorBuilder::new()
                        .with_source_opt(&e.src)
                        .with_destination("RsField")
                        .with_data(&value)
                        .with_error_source(e)
                        .with_span((&value.span()).into())
                        .build()
                })?;
                (name, ty)
            }
            FnArg::Receiver(_) => {
                return Err(ConversionErrorBuilder::new()
                    .with_source("FnArg")
                    .with_destination("RsField")
                    .with_data(&value)
                    .with_message("FnArg::Receiver is not supported")
                    .with_span((&value.span()).into())
                    .build());
            }
        };
        Ok(Self::new(name, ty))
    }
}

/// Represents an enum in Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsEnum {
    /// The name of the enum.
    pub name: String,
    /// The variants of the enum.
    pub variants: Vec<RsVariant>,
}

impl Display for RsEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variant_str = self
            .variants
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "enum {} ({})", self.name, variant_str)
    }
}

impl RsEnum {
    /// Creates a new enum.
    pub fn new(name: String, variants: Vec<RsVariant>) -> Self {
        Self { name, variants }
    }
}

impl From<RsEnum> for RsType {
    fn from(e: RsEnum) -> Self {
        Self::Enum(e)
    }
}

impl TryFrom<&ItemEnum> for RsEnum {
    type Error = ConversionError;

    fn try_from(value: &ItemEnum) -> Result<Self, Self::Error> {
        let name = value.ident.to_string();
        let variants = value
            .variants
            .iter()
            .map(RsVariant::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                ConversionErrorBuilder::new()
                    .with_source_opt(&e.src)
                    .with_destination("RsEnum")
                    .with_data(&value)
                    .with_error_source(e)
                    .with_span((&value.span()).into())
                    .build()
            })?;
        Ok(Self::new(name, variants))
    }
}

/// Represents a variant of an enum in Rust. See [RsEnum] for more information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsVariant {
    /// The name of the variant.
    pub name: String,
    /// The fields of the variant.
    pub fields: Vec<RsField>,
}

impl Display for RsVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let field_str = self
            .fields
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{} ({})", self.name, field_str)
    }
}

impl RsVariant {
    /// Creates a new variant.
    pub fn new(name: String, fields: Vec<RsField>) -> Self {
        Self { name, fields }
    }
}

impl TryFrom<&Variant> for RsVariant {
    type Error = ConversionError;

    fn try_from(value: &Variant) -> Result<Self, Self::Error> {
        let name = value.ident.to_string();
        let fields = value
            .fields
            .iter()
            .map(RsField::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                ConversionErrorBuilder::new()
                    .with_source_opt(&e.src)
                    .with_destination("RsVariant")
                    .with_data(&value)
                    .with_error_source(e)
                    .with_span((&value.span()).into())
                    .build()
            })?;
        Ok(Self::new(name, fields))
    }
}

/// Represents a function in Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsFn {
    /// The name of the function.
    pub name: String,
    /// The arguments of the function.
    pub args: Vec<RsField>,
    /// The return type of the function.
    pub ret: Option<Box<RsType>>,
}

impl Display for RsFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arg_str = self
            .args
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        if let Some(ret) = &self.ret {
            write!(f, "fn {}({}) -> {}", self.name, arg_str, ret)
        } else {
            write!(f, "fn {}({})", self.name, arg_str)
        }
    }
}

impl RsFn {
    /// Creates a new function.
    pub fn new(name: String, args: Vec<RsField>, ret: RsType) -> Self {
        Self {
            name,
            args,
            ret: Some(Box::new(ret)),
        }
    }
}

impl From<RsFn> for RsType {
    fn from(f: RsFn) -> Self {
        Self::Func(f)
    }
}

impl TryFrom<&ReturnType> for RsType {
    type Error = ConversionError;

    fn try_from(value: &ReturnType) -> Result<Self, Self::Error> {
        match value {
            ReturnType::Default => Ok(Self::Unit),
            ReturnType::Type(_, ty) => {
                RsType::try_from(ty.as_ref()).map_err(|e| {
                    ConversionErrorBuilder::new()
                        .with_source_opt(&e.src)
                        .with_destination("RsType")
                        .with_data(&value)
                        .with_error_source(e)
                        .with_span((&value.span()).into())
                        .build()
                })
            }
        }
    }
}

impl TryFrom<&ItemFn> for RsFn {
    type Error = ConversionError;

    fn try_from(value: &ItemFn) -> Result<Self, Self::Error> {
        let name = value.sig.ident.to_string();
        let args = value
            .sig
            .inputs
            .iter()
            .map(RsField::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                ConversionErrorBuilder::new()
                    .with_source_opt(&e.src)
                    .with_destination("RsFn")
                    .with_data(&value)
                    .with_error_source(e)
                    .with_span((&value.span()).into())
                    .build()
            })?;
        let ret = RsType::try_from(&value.sig.output).map_err(|e| {
            ConversionErrorBuilder::new()
                .with_source_opt(&e.src)
                .with_destination("RsFn")
                .with_data(&value)
                .with_error_source(e)
                .with_span((&value.span()).into())
                .build()
        })?;
        Ok(Self::new(name, args, ret))
    }
}

/// Represents an array in Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsArray {
    /// The type of the array.
    pub ty: Box<RsType>,
    /// The length of the array.
    pub len: usize,
}

impl Display for RsArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}; {}]", self.ty, self.len)
    }
}

impl RsArray {
    /// Creates a new array.
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

impl TryFrom<&TypeArray> for RsArray {
    type Error = ConversionError;

    fn try_from(value: &TypeArray) -> Result<Self, Self::Error> {
        let ty = RsType::try_from(value.elem.as_ref()).map_err(|e| {
            ConversionErrorBuilder::new()
                .with_source_opt(&e.src)
                .with_destination("RsArray")
                .with_data(&value)
                .with_error_source(e)
                .with_span((&value.span()).into())
                .build()
        })?;
        let len = value.len.value() as usize;
        Ok(Self::new(ty, len))
    }
}

/// Represents a primitive type in Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RsPrimitive {
    /// Represents the [i8] type in Rust.
    I8,
    /// Represents the [i16] type in Rust.
    I16,
    /// Represents the [i32] type in Rust.
    I32,
    /// Represents the [i64] type in Rust.
    I64,
    /// Represents the [i128] type in Rust.
    I128,
    /// Represents the [u8] type in Rust.
    U8,
    /// Represents the [u16] type in Rust.
    U16,
    /// Represents the [u32] type in Rust.
    U32,
    /// Represents the [u64] type in Rust.
    U64,
    /// Represents the [u128] type in Rust.
    U128,
    /// Represents the [f32] type in Rust.
    F32,
    /// Represents the [f64] type in Rust.
    F64,
    /// Represents the [bool] type in Rust.
    Bool,
    /// Represents the [char] type in Rust.
    Char,
    /// Represents the [str] type in Rust.
    Str,
    /// Represents the [String] type in Rust.
    String,
    /// Represents the [()] type in Rust.
    Unit,
}

impl Display for RsPrimitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RsPrimitive::I8 => write!(f, "i8"),
            RsPrimitive::I16 => write!(f, "i16"),
            RsPrimitive::I32 => write!(f, "i32"),
            RsPrimitive::I64 => write!(f, "i64"),
            RsPrimitive::I128 => write!(f, "i128"),
            RsPrimitive::U8 => write!(f, "u8"),
            RsPrimitive::U16 => write!(f, "u16"),
            RsPrimitive::U32 => write!(f, "u32"),
            RsPrimitive::U64 => write!(f, "u64"),
            RsPrimitive::U128 => write!(f, "u128"),
            RsPrimitive::F32 => write!(f, "f32"),
            RsPrimitive::F64 => write!(f, "f64"),
            RsPrimitive::Bool => write!(f, "bool"),
            RsPrimitive::Char => write!(f, "char"),
            RsPrimitive::Str => write!(f, "str"),
            RsPrimitive::String => write!(f, "String"),
            RsPrimitive::Unit => write!(f, "()"),
        }
    }
}

impl From<RsPrimitive> for RsType {
    fn from(p: RsPrimitive) -> Self {
        Self::Primitive(p)
    }
}

impl TryFrom<&TypePath> for RsPrimitive {
    type Error = ConversionError;

    fn try_from(value: &TypePath) -> Result<Self, Self::Error> {
        todo!()
    }
}

/// Represents a pointer in Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsPointer {
    /// The type of the object the pointer points to.
    pub ty: Box<RsType>,
    /// Whether the pointer is mutable or not.
    pub mutable: bool,
}

impl Display for RsPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.mutable {
            write!(f, "pointer (*mut {})", self.ty)
        } else {
            write!(f, "pointer (*const {})", self.ty)
        }
    }
}

impl RsPointer {
    /// Creates a new pointer.
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

impl TryFrom<&TypePtr> for RsPointer {
    type Error = ConversionError;

    fn try_from(value: &TypePtr) -> Result<Self, Self::Error> {
        todo!()
    }
}

/// Represents a tuple in Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsTuple {
    /// The types of the tuple.
    pub types: Vec<RsType>,
}

impl Display for RsTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_str = self
            .types
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "tuple ({})", type_str)
    }
}

impl RsTuple {
    /// Creates a new tuple.
    pub fn new(types: Vec<RsType>) -> Self {
        Self { types }
    }
}

impl From<RsTuple> for RsType {
    fn from(t: RsTuple) -> Self {
        Self::Tuple(t)
    }
}

impl TryFrom<&TypeTuple> for RsTuple {
    type Error = ConversionError;

    fn try_from(value: &TypeTuple) -> Result<Self, Self::Error> {
        todo!()
    }
}

/// Represents a slice in Rust.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsSlice {
    /// The type of the slice.
    pub ty: Box<RsType>,
}

impl Display for RsSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.ty)
    }
}

impl RsSlice {
    /// Creates a new slice.
    pub fn new(ty: RsType) -> Self {
        Self { ty: Box::new(ty) }
    }
}

impl From<RsSlice> for RsType {
    fn from(s: RsSlice) -> Self {
        Self::Slice(s)
    }
}

impl TryFrom<&TypeSlice> for RsSlice {
    type Error = ConversionError;

    fn try_from(value: &TypeSlice) -> Result<Self, Self::Error> {
        todo!()
    }
}
