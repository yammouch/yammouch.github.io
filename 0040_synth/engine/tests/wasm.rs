use wasm_bindgen_test::*;
use reson::log;

#[wasm_bindgen_test]
fn angle_regu_test() {
  assert_eq!(0., reson::angle_regu(0.));
}
