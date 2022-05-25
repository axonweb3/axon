SCHEMA_PATH := src/schemas
SCHEMA_DEST_PATH := src/generated

schema:
	moleculec --language rust --schema-file ${SCHEMA_PATH}/axon.mol > ${SCHEMA_DEST_PATH}/axon.rs
	moleculec --language rust --schema-file ${SCHEMA_PATH}/crosschain.mol > ${SCHEMA_DEST_PATH}/crosschain.rs
