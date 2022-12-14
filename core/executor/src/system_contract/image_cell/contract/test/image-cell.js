const { ethers } = require("hardhat");

describe("ImageCell", function () {
    let imageCell;

    before("deploy the contract instance first", async function () {
        const ImageCell = await ethers.getContractFactory("ImageCell");
        imageCell = await ImageCell.deploy();
        await imageCell.deployed();
    });

    it("should test args of setState()", async () => {
        await imageCell.setState(true);
    });

    it("should test args of update()", async () => {
        let header = {
            version: 0x0,
            compactTarget: 0x1a9c7b1a,
            timestamp: 0x16e62df76ed,
            number: 0x129d3,
            epoch: 0x7080291000049,
            parentHash: "0x815ecf2140169b9d283332c7550ce8b6405a120d5c21a7aa99d8a75eb9e77ead",
            transactionsRoot: "0x66ab0046436f97aefefe0549772bf36d96502d14ad736f7f4b1be8274420ca0f",
            proposalsHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
            unclesHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
            dao: "0x7088b3ee3e738900a9c257048aa129002cd43cd745100e000066ac8bd8850d00",
            nonce: 0x78b105de64fc38a200000004139b0200n,
            blockHash: "0x87764caf4a0e99302f1382421da1fe2f18382a49eac2d611220056b0854868e3"
        };

        let inputs = [{
            txHash: "0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37",
            index: 0x0,
        }];

        let outputs = [{
            outPoint: {
                txHash: "0x65b253cdcb6226e7f8cffec5c47c959b3d74af2caf7970a1eb1500e9b92aa200",
                index: 0x0,
            },
            output: {
                capacity: 0x34e62ce00,
                lock: {
                    args: "0x927f3e74dceb87c81ba65a19da4f098b4de75a0d",
                    codeHash: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
                    hashType: 1
                },
                type_: [{
                    args: "0x6e9b17739760ffc617017f157ed40641f7aa51b2af9ee017b35a0b35a1e2297b",
                    codeHash: "0x48dbf59b4c7ee1547238021b4869bceedf4eea6b43772e5d66ef8865b6ae7212",
                    hashType: 0
                }]
            },
            data: "0x40420f00000000000000000000000000"
        }];

        await imageCell.update(header, inputs, outputs);
    });

    it("should test args of rollback()", async () => {
        let blockHash = "0x87764caf4a0e99302f1382421da1fe2f18382a49eac2d611220056b0854868e3";
        let blockNumber = 0x129d3;

        let inputs = [{
            txHash: "0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37",
            index: 0x0,
        }];

        let outputs = [{
            txHash: "0x65b253cdcb6226e7f8cffec5c47c959b3d74af2caf7970a1eb1500e9b92aa200",
            index: 0x0,
        }];

        await imageCell.rollback(blockHash, blockNumber, inputs, outputs);
    });
});
