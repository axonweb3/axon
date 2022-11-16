use axon::{FeeAllocate, FeeInlet, ValidatorExtend, H160, U256};

#[derive(Default, Clone, Debug)]
struct CustomFeeAllocator;

impl FeeAllocate for CustomFeeAllocator {
    fn allocate(
        &self,
        _block_number: U256,
        _fee_collect: U256,
        _proposer: H160,
        _validators: &[ValidatorExtend],
    ) -> Vec<FeeInlet> {
        // Write your custom fee allocation process below.
        todo!()
    }
}

fn main() {
    axon::run(CustomFeeAllocator::default())
}
