#[derive(Debug, Clone, Copy)]
pub enum Cycle {
    ReadAtPc,
    ReadAtPcInc,
    ReadAtSp,
    ReadAtSpInc,
    ReadAtSpDec,

    PushPchDec,
    PushPclDec,
    PushPSetBDec,
    PushPDec,
    PushADec,

    PullPInc,
    PullP,
    PullA,
    PullPclInc,
    PullPch,

    FetchAdlIncPc,
    FetchAdhApplyPc,

    FetchResVcl,
    FetchResVch,
    FetchBrkVcl, // TODO: Emulate vector hijack
    FetchBrkVch,
}

use Cycle::*;

#[rustfmt::skip]
pub const RESET: &[Cycle] = &[ReadAtPc, ReadAtSpDec, ReadAtSpDec, ReadAtSpDec, FetchResVcl, FetchResVch];

#[rustfmt::skip]
pub const BRK: &[Cycle] = &[ReadAtPcInc, PushPchDec, PushPclDec, PushPSetBDec, FetchBrkVcl, FetchBrkVch];

pub const RTI: &[Cycle] = &[ReadAtPc, ReadAtSpInc, PullPInc, PullPclInc, PullPch];
pub const RTS: &[Cycle] = &[ReadAtPc, ReadAtSpInc, PullPclInc, PullPch, ReadAtPcInc];

pub const PHP: &[Cycle] = &[ReadAtPc, PushPDec];
pub const PHA: &[Cycle] = &[ReadAtPc, PushADec];
pub const PLP: &[Cycle] = &[ReadAtPc, ReadAtSpInc, PullP];
pub const PLA: &[Cycle] = &[ReadAtPc, ReadAtSpInc, PullA];

#[rustfmt::skip]
pub const JSR_ABS: &[Cycle] = &[FetchAdlIncPc, ReadAtSp, PushPchDec, PushPclDec, FetchAdhApplyPc];
pub const JMP_ABS: &[Cycle] = &[FetchAdlIncPc, FetchAdhApplyPc];
pub const JMP_IND: &[Cycle] = &[];
