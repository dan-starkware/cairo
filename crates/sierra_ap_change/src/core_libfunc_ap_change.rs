use sierra::extensions::array::ArrayConcreteLibFunc;
use sierra::extensions::core::CoreConcreteLibFunc;
use sierra::extensions::dict_felt_to::DictFeltToConcreteLibFunc;
use sierra::extensions::enm::EnumConcreteLibFunc;
use sierra::extensions::felt::FeltConcrete;
use sierra::extensions::gas::GasConcreteLibFunc;
use sierra::extensions::integer::{IntOperator, Uint128Concrete, Uint128OperationConcreteLibFunc};
use sierra::extensions::mem::MemConcreteLibFunc;
use sierra::extensions::starknet::StarkNetConcreteLibFunc;
use sierra::extensions::strct::StructConcreteLibFunc;

use crate::ApChange;

/// Returns the ap change for a core libfunc.
/// Values with unknown values will return as None.
pub fn core_libfunc_ap_change(libfunc: &CoreConcreteLibFunc) -> Vec<ApChange> {
    match libfunc {
        CoreConcreteLibFunc::ApTracking(_) => vec![ApChange::Unknown],
        CoreConcreteLibFunc::Array(libfunc) => match libfunc {
            ArrayConcreteLibFunc::New(_) => vec![ApChange::Known(1)],
            ArrayConcreteLibFunc::Append(_) => vec![ApChange::Known(0)],
            ArrayConcreteLibFunc::At(_) => vec![ApChange::Known(5), ApChange::Known(3)],
            ArrayConcreteLibFunc::Len(_) => vec![ApChange::Known(0)],
        },
        // TODO(orizi): Make this variable dependent.
        CoreConcreteLibFunc::BranchAlign(_) => vec![ApChange::Known(0)],
        CoreConcreteLibFunc::Box(_) => vec![ApChange::Known(0)],
        // TODO(lior): Check/Fix.
        CoreConcreteLibFunc::BuiltinCost(_) => vec![ApChange::Known(2), ApChange::Known(3)],
        CoreConcreteLibFunc::Drop(_) | CoreConcreteLibFunc::Dup(_) => vec![ApChange::Known(0)],
        CoreConcreteLibFunc::Felt(libfunc) => match libfunc {
            FeltConcrete::BinaryOperation(_)
            | FeltConcrete::Const(_)
            | FeltConcrete::UnaryOperation(_) => vec![ApChange::Known(0)],
            FeltConcrete::JumpNotZero(_) => vec![ApChange::Known(0), ApChange::Known(0)],
        },
        CoreConcreteLibFunc::FunctionCall(libfunc) => {
            vec![ApChange::FunctionCall(libfunc.function.id.clone())]
        }
        CoreConcreteLibFunc::Gas(libfunc) => match libfunc {
            GasConcreteLibFunc::GetGas(_) => vec![ApChange::Known(2), ApChange::Known(3)],
            GasConcreteLibFunc::RefundGas(_) => vec![ApChange::Known(0)],
        },
        CoreConcreteLibFunc::Uint128(libfunc) => match libfunc {
            Uint128Concrete::Operation(libfunc) => match libfunc {
                Uint128OperationConcreteLibFunc::Binary(libfunc) => match libfunc.operator {
                    IntOperator::OverflowingAdd | IntOperator::OverflowingSub => {
                        vec![ApChange::Known(2), ApChange::Known(3)]
                    }
                    IntOperator::OverflowingMul => todo!(),
                    IntOperator::DivMod => vec![ApChange::Known(5)],
                },
                Uint128OperationConcreteLibFunc::Const(_) => todo!(),
            },
            Uint128Concrete::LessThan(_) => vec![ApChange::Known(2), ApChange::Known(3)],
            Uint128Concrete::LessThanOrEqual(_) => vec![ApChange::Known(3), ApChange::Known(2)],
            Uint128Concrete::FromFelt(_) => vec![ApChange::Known(1), ApChange::Known(6)],
            Uint128Concrete::Const(_) | Uint128Concrete::ToFelt(_) => vec![ApChange::Known(0)],
            Uint128Concrete::JumpNotZero(_) => vec![ApChange::Known(0), ApChange::Known(0)],
        },
        CoreConcreteLibFunc::Mem(libfunc) => match libfunc {
            MemConcreteLibFunc::StoreTemp(libfunc) => {
                vec![ApChange::KnownByTypeSize(libfunc.ty.clone())]
            }
            MemConcreteLibFunc::AlignTemps(libfunc) => {
                vec![ApChange::KnownByTypeSize(libfunc.ty.clone())]
            }
            MemConcreteLibFunc::StoreLocal(_) => vec![ApChange::Known(0)],
            MemConcreteLibFunc::FinalizeLocals(_) => vec![ApChange::FinalizeLocals],
            MemConcreteLibFunc::AllocLocal(_) | MemConcreteLibFunc::Rename(_) => {
                vec![ApChange::Known(0)]
            }
        },
        CoreConcreteLibFunc::UnwrapNonZero(_) => vec![ApChange::Known(0)],
        CoreConcreteLibFunc::UnconditionalJump(_) => vec![ApChange::Known(0)],
        CoreConcreteLibFunc::Enum(libfunc) => match libfunc {
            EnumConcreteLibFunc::Init(_) => vec![ApChange::Known(0)],
            EnumConcreteLibFunc::Match(libfunc) => {
                vec![ApChange::Known(0); libfunc.signature.branch_signatures.len()]
            }
        },
        CoreConcreteLibFunc::Struct(libfunc) => match libfunc {
            StructConcreteLibFunc::Construct(_) | StructConcreteLibFunc::Deconstruct(_) => {
                vec![ApChange::Known(0)]
            }
        },
        CoreConcreteLibFunc::DictFeltTo(libfunc) => match libfunc {
            DictFeltToConcreteLibFunc::New(_) => vec![ApChange::Known(1)],
            DictFeltToConcreteLibFunc::Read(_) => vec![ApChange::Known(1)],
            DictFeltToConcreteLibFunc::Write(_) => vec![ApChange::Known(1)],
            DictFeltToConcreteLibFunc::Squash(_) => vec![ApChange::Unknown],
        },
        CoreConcreteLibFunc::Pedersen(_) => vec![ApChange::Known(0)],
        CoreConcreteLibFunc::StarkNet(libfunc) => match libfunc {
            StarkNetConcreteLibFunc::StorageRead(_) => vec![ApChange::Known(1)],
            StarkNetConcreteLibFunc::StorageAddressConst(_) => vec![ApChange::Known(0)],
        },
    }
}
