/*******************************************************************************
 * Copyright (c) 2018-2019 Aion foundation.
 *
 *     This file is part of the aion network project.
 *
 *     The aion network project is free software: you can redistribute it
 *     and/or modify it under the terms of the GNU General Public License
 *     as published by the Free Software Foundation, either version 3 of
 *     the License, or any later version.
 *
 *     The aion network project is distributed in the hope that it will
 *     be useful, but WITHOUT ANY WARRANTY; without even the implied
 *     warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 *     See the GNU General Public License for more details.
 *
 *     You should have received a copy of the GNU General Public License
 *     along with the aion network project source files.
 *     If not, see <https://www.gnu.org/licenses/>.
 *
 ******************************************************************************/

extern crate num_bigint;
extern crate bigdecimal;
extern crate num_traits;
#[macro_use]
extern crate lazy_static;

mod log_approximator;

use std::fmt;

use num_bigint::{BigUint,ToBigInt};
use bigdecimal::BigDecimal;
use num_traits::{Zero,One,Num};

//pub use

const PRECISION: usize = 70;
//const MAX_PRECISION : BigUint = BigUint::one().shl(PRECISION);

#[derive(Debug)]
pub enum FixedPointError {
    Negative,
}

impl fmt::Display for FixedPointError {
    fn fmt(&self, f: &mut fmt::Formatter) -> ::std::fmt::Result {
        f.write_str(match *self {
            FixedPointError::Negative => "Should not be negative",
        })
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct FixedPoint(BigUint);

impl FixedPoint {
    pub fn new() -> FixedPoint { Self::zero() }

    pub fn zero() -> FixedPoint { FixedPoint(BigUint::zero()) }

    pub fn one() -> FixedPoint { FixedPoint(BigUint::one() << PRECISION) }

    pub fn parse_from_big_decimal(num: BigDecimal) -> Result<FixedPoint, FixedPointError> {
        let temp = (num * FixedPoint::one().0.to_bigint().unwrap())
            .to_bigint()
            .unwrap();
        if temp > Zero::zero() {
            Ok(FixedPoint(temp.to_biguint().unwrap()))
        } else {
            Err(FixedPointError::Negative)
        }
    }

    pub fn add(&self, addend: FixedPoint) -> FixedPoint {
        let res = &self.0 + addend.0;
        FixedPoint(res)
    }

    pub fn subtruct(&self, subtructend: FixedPoint) -> Result<FixedPoint, FixedPointError> {
        if self.0 < subtructend.0 {
            Err(FixedPointError::Negative)
        } else {
            Ok(FixedPoint(&self.0 - subtructend.0))
        }
    }

    //    pub fn multiply(&self, multiplicand: FixedPoint) -> FixedPoint {
    //        FixedPoint(&self.0 * multiplicand.0)
    //    }

    pub fn multiply_uint(&self, multiplicand: BigUint) -> FixedPoint {
        FixedPoint(&self.0 * multiplicand)
    }

    pub fn divide_uint(&self, divisor: BigUint) -> FixedPoint { FixedPoint(&self.0 / divisor) }

    pub fn divide_by_power_of_two(&self, shift: usize) -> FixedPoint {
        FixedPoint(&self.0 >> shift)
    }
}
//impl Zero for FixedPoint{
//    fn zero()-> FixedPoint {
//        FixedPoint(BigUint::zero())
//    }
//}
//
//impl One for FixedPoint{
//    fn one()-> FixedPoint {
//        FixedPoint(MAX_PRECISION)
//    }
//}

impl From<BigUint> for FixedPoint {
    fn from(value: BigUint) -> FixedPoint { FixedPoint(value) }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        let fixed33 = FixedPoint(BigUint::from(33u64));
        let fixed47 = FixedPoint(BigUint::from(47u64));
        assert_eq!(fixed33.add(fixed47), FixedPoint(BigUint::from(80u64)));
    }

    #[test]
    fn test_parse_from_big_decimal() {
        let n1 = BigDecimal::from_str_radix("0.40546510810816438486", 10).unwrap();
        let fixed_n1 = FixedPoint::parse_from_big_decimal(n1).unwrap();

        assert_eq!(
            fixed_n1,
            FixedPoint(BigUint::parse_bytes(b"478688709125778178049", 10).unwrap())
        );

        let n2 = BigDecimal::from_str_radix("-0.40546510810816438486", 10).unwrap();
        let fixed_n2 = FixedPoint::parse_from_big_decimal(n2);

        assert!(fixed_n2.is_err());
    }

    #[test]
    fn test_subtruct() {
        let fixed50 = FixedPoint(BigUint::from(50u64));
        let fixed60 = FixedPoint(BigUint::from(60u64));
        let fixed40 = FixedPoint(BigUint::from(40u64));
        assert_eq!(
            fixed50.subtruct(fixed40).unwrap(),
            FixedPoint(BigUint::from(10u64))
        );
        assert!(fixed50.subtruct(fixed60).is_err());
    }

    #[test]
    fn test_multiply_uint() {
        let fixed50 = FixedPoint(BigUint::from(50u64));
        assert_eq!(
            fixed50.multiply_uint(BigUint::from(50u64)),
            FixedPoint(BigUint::from(2500u64))
        )
    }

    #[test]
    fn test_divide_uint() {
        let fixed50 = FixedPoint(BigUint::from(50u64));
        assert_eq!(
            fixed50.divide_uint(BigUint::from(2u64)),
            FixedPoint(BigUint::from(25u64))
        )
    }

    #[test]
    fn test_divide_by_power_of_two() {
        let fixed50 = FixedPoint(BigUint::from(50u64));
        assert_eq!(
            fixed50.divide_by_power_of_two(4),
            FixedPoint(BigUint::from(3u64))
        )
    }
}
