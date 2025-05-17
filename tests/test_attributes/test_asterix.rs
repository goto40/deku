//! This test show how you can implement Data Items defined
//! in the [ASTERIX Standard](https://www.eurocontrol.int/sites/default/files/2021-11/eurocontrol-specification-asterix-part1-ed-3-1.pdf)..Default::default()
//!
//! The [asterix crate](https://docs.rs/asterix/latest/asterix/)
//! gives an initial motivation how to use deku for ASTERIX.
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
    struct FixedLengthItem {
        #[deku(bits = "5")]
        pub uint5_example: u8,
        #[deku(bits = "9")]
        pub sint9_example: i16,
        #[deku(bits = "18")]
        pub fix18_example: i32, // fixpoint number (here: 1bit == 0.01)
    }
    impl FixedLengthItem {
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
    let mut decoded = FixedLengthItem::from_bytes((&raw_bytes, 0)).unwrap().1;
    assert_eq!(decoded.uint5_example, 4 + 1);
    assert_eq!(decoded.sint9_example, -2);
    assert_eq!(decoded.fix18_example, 7);
    assert!((decoded.get_fix18_example() - 0.07).abs() < 1e-5);
    decoded.set_fix18_example(5.02);
    assert_eq!(decoded.fix18_example, 502);
}

#[test]
fn test_variable_length_data_item() {
    #[derive(Debug, PartialEq, DekuRead, DekuWrite)]
    #[deku(endian = "big", bit_order = "msb")]
    struct Part1 {
        #[deku(bits = "5")]
        pub uint5_example: u8,
        #[deku(bits = "9")]
        pub sint9_example: i16,
        #[deku(bits = "17")]
        pub sint17_example: i32,
    }
    #[derive(Debug, PartialEq, Clone, DekuRead, DekuWrite)]
    #[deku(endian = "big", bit_order = "msb")]
    struct Part2 {
        #[deku(bits = "4")]
        pub uint5_example: u8,
        #[deku(bits = "8")]
        pub sint7_example: i8,
        #[deku(bits = "3")]
        pub uint2_example: u8,
        #[deku(bits = "1")]
        fx: u8, // non-pub
    }

    impl Part2 {
        pub fn new(uint5_example: u8, sint7_example: i8, uint2_example: u8) -> Self {
            Self {
                uint5_example,
                sint7_example,
                uint2_example,
                fx: 0,
            }
        }
    }

    #[deku_derive(DekuRead, DekuWrite)]
    #[derive(Debug, PartialEq)]
    struct VariableLengthItem {
        part1: Part1,
        #[deku(bits = "1", temp, temp_value = "if (*part2).is_empty() {0} else {1}")]
        pub part1_fx: u8,
        #[deku(
            skip,
            cond = "*part1_fx==0",
            default = "Default::default()",
            until = "|codefx: &Part2| codefx.fx == 0",
            update = "self.part2.iter_mut().rev().enumerate().map(|(index, entry)| { entry.fx = if index!=0 {1} else {0}; entry.clone() }).rev().collect::<Vec<_>>()"
        )]
        part2: Vec<Part2>,
    }

    let raw_bytes = vec![
        0b_00101_111,
        0b111110_00,
        0x00,
        0x0f,
        0xaf,
        0xff,
        0xb0,
        0x00,
    ];
    let mut decoded = VariableLengthItem::from_bytes((&raw_bytes, 0)).unwrap().1;
    assert_eq!(decoded.part1.uint5_example, 4 + 1);
    assert_eq!(decoded.part1.sint9_example, -2);
    assert_eq!(decoded.part1.sint17_example, 7);

    assert_eq!(decoded.part2.len(), 2);
    assert_eq!(decoded.part2[0].uint5_example, 0xa);
    assert_eq!(decoded.part2[1].uint5_example, 0xb);

    assert_eq!(decoded.part2[0].fx, 1);
    assert_eq!(decoded.part2[1].fx, 0);

    let raw_bytes_reproduced = decoded.to_bytes().unwrap();
    assert_eq!(raw_bytes_reproduced, raw_bytes);

    decoded.part2.push(Part2::new(0, -1, 3));
    decoded.part2.push(Part2::new(1, -1, 3));
    decoded.part2.push(Part2::new(2, -1, 3));
    decoded.update().unwrap();

    let raw_bytes_produced = decoded.to_bytes().unwrap();

    let decoded2 = VariableLengthItem::from_bytes((&raw_bytes_produced, 0))
        .unwrap()
        .1;
    assert_eq!(decoded, decoded2);
}
