//! This test show how you can implement Data Items defined
//! in the [ASTERIX Standard](https://www.eurocontrol.int/sites/default/files/2021-11/eurocontrol-specification-asterix-part1-ed-3-1.pdf)..Default::default()
//!
//! Messages in ASTERIX consist of Records, with a logic depicted in [test_temp_value_with_cond.rs](./test_temp_value_with_cond.rs)..Default::default()
//! Within these messages you find 5 types of Data Items:
//!
//! - Fixed Length Data Items
//! - Extended Length Data Items
//! - Explicit Length Data Items
//! - Repetitive Data Items
//! - Compound Data Items
//!
//! This unit test gives initial hints in how to implement these.
//!

use deku::prelude::*;

#[test]
fn test_fixed_length_data_item() {
    #[derive(Debug, PartialEq, DekuRead, DekuWrite)]
    #[deku(endian = "big", bit_order = "msb")]
    struct TestStruct {
        #[deku(bits = "5")]
        pub uint5_example: u8,
        #[deku(bits = "9")]
        pub sint9_example: i16,
        #[deku(bits = "18")]
        pub fix18_example: i32, // fixpoint number (here: 1bit == 0.01)
    }
    impl TestStruct {
        pub fn get_fix18_example(&self) -> f32 {
            (self.fix18_example as f32) * 0.01
        }
        pub fn set_fix18_example(&mut self, v: f32) {
            // better use a create like FloatToInt,
            // also you need to handle over-/underflows
            self.fix18_example = (v / 0.01).round() as i32;
        }
    }

    let raw_bytes = vec![0b_00101_111, 0b111110_00, 0x00, 0x07];
    let mut decoded = TestStruct::from_bytes((&raw_bytes, 0)).unwrap().1;
    assert_eq!(decoded.uint5_example, 4 + 1);
    assert_eq!(decoded.sint9_example, -2);
    assert_eq!(decoded.fix18_example, 7);
    assert!((decoded.get_fix18_example() - 0.07).abs() < 1e-5);
    decoded.set_fix18_example(5.02);
    assert_eq!(decoded.fix18_example, 502);
}
