use ckb_sdk::rpc::ckb_indexer as indexer;
use ckb_types::{bytes::Bytes, packed, prelude::*};

#[derive(Debug)]
pub struct Cell {
    pub output:       packed::CellOutput,
    pub output_data:  Bytes,
    pub out_point:    packed::OutPoint,
    pub block_number: u64,
    pub tx_index:     u32,
}

impl From<indexer::Cell> for Cell {
    fn from(json_cell: indexer::Cell) -> Self {
        Self::from(&json_cell)
    }
}

impl From<&indexer::Cell> for Cell {
    fn from(json_cell: &indexer::Cell) -> Self {
        let output = {
            let output: packed::CellOutput = json_cell.output.clone().into();
            packed::CellOutput::new_unchecked(output.as_bytes())
        };
        let out_point = {
            let out_point: packed::OutPoint = json_cell.out_point.clone().into();
            packed::OutPoint::new_unchecked(out_point.as_bytes())
        };
        let block_number = u64::from(json_cell.block_number);
        let tx_index = u32::from(json_cell.tx_index);
        let output_data = json_cell
            .output_data
            .clone()
            .unwrap_or_default()
            .into_bytes();

        Cell {
            output,
            out_point,
            block_number,
            tx_index,
            output_data,
        }
    }
}
