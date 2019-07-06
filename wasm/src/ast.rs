#[derive(Clone, Debug, PartialEq)]
enum ValueType {
    I32,
    I63,
    F32,
    F64,
}

#[derive(Clone, Debug, PartialEq)]
struct BlockType(Option<ValueType>);

#[derive(Clone, Debug, PartialEq)]
enum ElemType {
    AnyFunc,
}

#[derive(Clone, Debug, PartialEq)]
struct FuncType {
    params: Vec<ValueType>,
    result: Option<ValueType>,
}

#[derive(Clone, Debug, PartialEq)]
enum LanguageType {
    ValueType(ValueType),
    ElemType(ElemType),
    FuncType(FuncType),
    BlockType(BlockType),
}

#[derive(Clone, Debug, PartialEq)]
struct GlobalType {
    content_type: ValueType,
    mutability: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct TableType {
    element_type: ElemType,
    limits: ResizableLimits,
}

#[derive(Clone, Debug, PartialEq)]
struct MemoryType(ResizableLimits);

#[derive(Clone, Debug, PartialEq)]
enum ExternalKind {
    Function,
    Table,
    Memory,
    Global,
}

#[derive(Clone, Debug, PartialEq)]
enum ExternalKindImport {
    Function(usize),
    Table(TableType),
    Memory(MemoryType),
    Global(GlobalType),
}

#[derive(Clone, Debug, PartialEq)]
struct ResizableLimits {
    initial: i32,
    maximum: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
enum InitExpr {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Global(usize),
}

#[derive(Clone, Debug, PartialEq)]
struct TypeSection(Vec<FuncType>);

#[derive(Clone, Debug, PartialEq)]
struct ImportEntry {
    module: String,
    field: String,
    kind: ExternalKindImport,
}

#[derive(Clone, Debug, PartialEq)]
struct ImportSection(Vec<ImportEntry>);

#[derive(Clone, Debug, PartialEq)]
struct FunctionSection(Vec<usize>);

#[derive(Clone, Debug, PartialEq)]
struct TableSection(Vec<TableType>);

#[derive(Clone, Debug, PartialEq)]
struct MemorySection(Vec<MemoryType>);

#[derive(Clone, Debug, PartialEq)]
struct GlobalSection(Vec<GlobalVariable>);

#[derive(Clone, Debug, PartialEq)]
struct GlobalVariable(GlobalType, InitExpr);

#[derive(Clone, Debug, PartialEq)]
struct ExportSection(Vec<ExportEntry>);

#[derive(Clone, Debug, PartialEq)]
struct ExportEntry {
    field: String,
    kind: ExternalKind,
    index: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct StartSection(usize);

#[derive(Clone, Debug, PartialEq)]
struct ElementSection(Vec<ElemSegment>);

#[derive(Clone, Debug, PartialEq)]
struct ElemSegment {
    offset: InitExpr,
    elems: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct CodeSection(Vec<FunctionBody>);

#[derive(Clone, Debug, PartialEq)]
struct FunctionBody {
    locals: Vec<LocalEntry>,
    codes: Vec<OperatorCode>,
}

#[derive(Clone, Debug, PartialEq)]
struct LocalEntry {
    count: usize,
    typ: ValueType,
}

#[derive(Clone, Debug, PartialEq)]
struct DataSection(Vec<DataSegment>);

#[derive(Clone, Debug, PartialEq)]
struct DataSegment {
    offset: InitExpr,
    data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
struct MemoryImmediate {
    flags: u32,
    offset: u32,
}

#[derive(Clone, Debug, PartialEq)]
enum OperatorCode {
    Unreachable,
    Nop,
    Block(BlockType),
    Loop(BlockType),
    If(BlockType),
    Else,
    End,
    Br(usize),
    BrIf(usize),
    BrTable { index: usize, params: Vec<usize> },
    Return,
    Call(usize),
    CallIndirect(usize),
    Drop,
    Select,
    GetLocal(usize),
    SetLocal(usize),
    TeeLocal(usize),
    GetGlobal(usize),
    SetGlobal(usize),
    I32Load(MemoryImmediate),
    I64Load(MemoryImmediate),
    F32Load(MemoryImmediate),
    F64Load(MemoryImmediate),
    I32Load8s(MemoryImmediate),
    I32Load8u(MemoryImmediate),
    I32Load16s(MemoryImmediate),
    I32Load16u(MemoryImmediate),
    I64Load8s(MemoryImmediate),
    I64Load8u(MemoryImmediate),
    I64Load16s(MemoryImmediate),
    I64Load16u(MemoryImmediate),
    I64Load32s(MemoryImmediate),
    I64Load32u(MemoryImmediate),
    I32Store(MemoryImmediate),
    I64Store(MemoryImmediate),
    F32Store(MemoryImmediate),
    F64Store(MemoryImmediate),
    I32Store8(MemoryImmediate),
    I32Store16(MemoryImmediate),
    I64Store8(MemoryImmediate),
    I64Store16(MemoryImmediate),
    I64Store32(MemoryImmediate),
    CurrentMemory,
    GrowMemory,
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    I32Eqz,
    I32Eq,
    I32Ne,
    I32Lts,
    I32Ltu,
    I32Gts,
    I32Gtu,
    I32Les,
    I32Leu,
    I32Ges,
    I32Geu,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64Lts,
    I64Ltu,
    I64Gts,
    I64Gtu,
    I64Les,
    I64Leu,
    I64Ges,
    I64Geu,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32Divs,
    I32Divu,
    I32Rems,
    I32Remu,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32Shrs,
    I32Shru,
    I32Rotl,
    I32Rotr,
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64Divs,
    I64Divu,
    I64Rems,
    I64Remu,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64Shrs,
    I64Shru,
    I64Rotl,
    I64Rotr,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,
    I32WrapI64,
    I32TruncsF32,
    I32TrancuF32,
    I32TrancsF64,
    I32TrancuF64,
    I64ExtendsI32,
    I64ExtenduI32,
    I64TruncsF32,
    I64TrancuF32,
    I64TrancsF64,
    I64TrancuF64,
    F32ConvertsI32,
    F32ConvertuI32,
    F32ConvertsI64,
    F32ConvertuI64,
    F32DemoteF64,
    F64ConvertsI32,
    F64ConvertuI32,
    F64ConvertsI64,
    F64ConvertuI64,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
}

#[derive(Clone, Debug, PartialEq, Default)]
struct WasmASTRoot {
    type_section: Option<TypeSection>,
    import_section: Option<ImportSection>,
    function_section: Option<FunctionSection>,
    table_section: Option<TableSection>,
    memory_section: Option<MemorySection>,
    global_section: Option<GlobalSection>,
    export_section: Option<ExportSection>,
    start_section: Option<StartSection>,
    element_section: Option<ElementSection>,
    code_section: Option<CodeSection>,
    data_section: Option<DataSection>,
}
