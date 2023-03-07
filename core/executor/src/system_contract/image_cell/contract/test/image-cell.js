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

        await imageCell.update(0x129d3, inputs, outputs);
    });

    it("should test args of rollback()", async () => {
        let inputs = [{
            txHash: "0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37",
            index: 0x0,
        }];

        let outputs = [{
            txHash: "0x65b253cdcb6226e7f8cffec5c47c959b3d74af2caf7970a1eb1500e9b92aa200",
            index: 0x0,
        }];

        await imageCell.rollback(inputs, outputs);
    });
});
