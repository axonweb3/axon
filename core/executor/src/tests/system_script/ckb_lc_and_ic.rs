use super::{ckb_light_client, image_cell};

#[test]
fn test_ckb_light_client_and_image_cell() {
    ckb_light_client::test_write_functions();
    image_cell::test_write_functions();
}
