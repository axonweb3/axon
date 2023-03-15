import subprocess


def main():
    cmd_args = [
        [
            "MetadataContract",
            "../../core/executor/src/system_contract/metadata/abi/metadata_abi.json",
            "../../core/executor/src/system_contract/metadata/abi/metadata_abi.rs",
        ],
        [
            "CkbLightClientContract",
            "../../core/executor/src/system_contract/ckb_light_client/abi/ckb_light_client_abi.json",
            "../../core/executor/src/system_contract/ckb_light_client/abi/ckb_light_client_abi.rs",
        ],
        [
            "ImageCellContract",
            "../../core/executor/src/system_contract/image_cell/abi/image_cell_abi.json",
            "../../core/executor/src/system_contract/image_cell/abi/image_cell_abi.rs",
        ],
    ]
    cargo_run = "cargo run --".split(" ")
    for args in cmd_args:
        subprocess.Popen(cargo_run + ["-c", args[0], "-j", args[1], "-o", args[2]])


if __name__ == "__main__":
    main()
