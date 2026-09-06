// CodeGen utility constants and helper functions for RISC-V 64

// Immediate value constraints
pub const IMM_12_MAX: i32 = 0x7FF;
pub const IMM_12_MIN: i32 = -0x800;

pub const LOW_12_MASK: u32 = 0x00000FFF;
pub const LOW_20_MASK: u32 = 0x000FFFFF;
pub const LOW_32_MASK: u64 = 0xFFFFFFFF;

pub fn align(x: usize, alignment: usize) -> usize {
    (x + (alignment - 1)) & !(alignment - 1)
}

pub fn is_imm_12(x: i32) -> bool {
    x <= IMM_12_MAX && x >= IMM_12_MIN
}

// Stack frame related
pub const PROLOGUE_OFFSET_BASE: usize = 16; // ra, fp
pub const PROLOGUE_ALIGN: usize = 16;

// RISC-V RV64 instructions
// Arithmetic
pub const ADD: &str = "add";
pub const SUB: &str = "sub";
pub const MUL: &str = "mul";
pub const DIV: &str = "div";

pub const ADDI: &str = "addi";
pub const ADDIW: &str = "addiw";

pub const FADD: &str = "fadd.s";
pub const FSUB: &str = "fsub.s";
pub const FMUL: &str = "fmul.s";
pub const FDIV: &str = "fdiv.s";

// Load upper immediate
pub const LUI: &str = "lui";
pub const AUIPC: &str = "auipc";

// Memory access
pub const LB: &str = "lb";
pub const LH: &str = "lh";
pub const LW: &str = "lw";
pub const LD: &str = "ld";

pub const SB: &str = "sb";
pub const SH: &str = "sh";
pub const SW: &str = "sw";
pub const SD: &str = "sd";

pub const FLW: &str = "flw";
pub const FSW: &str = "fsw";

// Branches
pub const BEQ: &str = "beq";
pub const BNE: &str = "bne";
pub const BLT: &str = "blt";
pub const BGE: &str = "bge";
pub const BLTU: &str = "bltu";
pub const BGEU: &str = "bgeu";

// Jumps
pub const JAL: &str = "jal";
pub const JALR: &str = "jalr";
pub const J: &str = "j";
pub const JR: &str = "jr";
pub const CALL: &str = "call";
pub const RET: &str = "ret";

// Comparisons
pub const SLT: &str = "slt";
pub const SLTU: &str = "sltu";
pub const SLTI: &str = "slti";
pub const SLTIU: &str = "sltiu";

// Logical
pub const AND: &str = "and";
pub const OR: &str = "or";
pub const XOR: &str = "xor";
pub const ANDI: &str = "andi";
pub const ORI: &str = "ori";
pub const XORI: &str = "xori";

// Shifts
pub const SLL: &str = "sll";
pub const SRL: &str = "srl";
pub const SRA: &str = "sra";

// Float comparisons
pub const FEQ: &str = "feq.s";
pub const FLT_CMP: &str = "flt.s";
pub const FLE: &str = "fle.s";

// Float conversions
pub const FCVT_W_S: &str = "fcvt.w.s";
pub const FCVT_S_W: &str = "fcvt.s.w";

// Float move
pub const FMV_W_X: &str = "fmv.w.x";
pub const FMV_X_W: &str = "fmv.x.w";

// Pseudo-instructions
pub const LI: &str = "li";
pub const LA: &str = "la";
pub const MV: &str = "mv";
pub const SEQZ: &str = "seqz";
pub const SNEZ: &str = "snez";
