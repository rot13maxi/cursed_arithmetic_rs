use cursed_arithmetic::{BigNum, Digit};

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }
}

fn random_bignum(rng: &mut Lcg, max_len: usize) -> BigNum {
    let len = 1 + (rng.next() as usize % max_len);
    let mut digits = Vec::with_capacity(len);
    let first = (rng.next() % 9 + 1) as u8;
    digits.push(Digit::from_value(first).unwrap());
    for _ in 1..len {
        digits.push(Digit::from_value((rng.next() % 10) as u8).unwrap());
    }
    BigNum::from(digits)
}

#[test]
fn small_arithmetic_matches_u128() {
    let mut rng = Lcg(0xC0FF_EE00_1234_5678);

    for _ in 0..2_000 {
        let a = rng.next();
        let b = rng.next();
        let ba = BigNum::from(a);
        let bb = BigNum::from(b);

        assert_eq!((&ba + &bb).to_u128(), Some(u128::from(a) + u128::from(b)));
        assert_eq!((&ba * &bb).to_u128(), Some(u128::from(a) * u128::from(b)));

        let (larger, smaller) = if a >= b { (a, b) } else { (b, a) };
        assert_eq!(
            (BigNum::from(larger) - BigNum::from(smaller)).to_u128(),
            Some(u128::from(larger - smaller))
        );

        if b != 0 {
            assert_eq!((&ba / &bb).to_u128(), a.checked_div(b).map(u128::from));
            assert_eq!((&ba % &bb).to_u128(), a.checked_rem(b).map(u128::from));
        }
    }
}

#[test]
fn large_arithmetic_is_self_consistent() {
    let mut rng = Lcg(0xDEAD_BEEF_CAFE_F00D);

    for _ in 0..200 {
        let a = random_bignum(&mut rng, 80);
        let b = random_bignum(&mut rng, 80);
        let c = random_bignum(&mut rng, 40);

        assert_eq!(a.to_string().parse::<BigNum>().unwrap(), a);
        assert_eq!(&a + &b, &b + &a);
        assert_eq!(&a * &b, &b * &a);

        let sum = &a + &b;
        assert_eq!(&sum - &b, a);

        let product = &a * &b;
        assert_eq!(product, &b * &a);

        let (quotient, remainder) = product.div_rem(&b);
        assert_eq!(quotient, a);
        assert!(remainder.is_zero());

        assert_eq!(&a * (&b + &c), &a * &b + &a * &c);
        assert_eq!((&a + &b) - &a, b);

        let (quotient, remainder) = a.div_rem(&a);
        assert_eq!(quotient.to_string(), "1");
        assert!(remainder.is_zero());

        assert_eq!(&a / &BigNum::one(), a);
        assert_eq!(&a % &BigNum::one(), BigNum::zero());
        assert_eq!(&a * &BigNum::zero(), BigNum::zero());
    }
}

#[test]
fn u128_boundaries_round_trip() {
    for value in [0, 1, 9, 10, u64::MAX as u128, u128::MAX] {
        let number = BigNum::from(value);
        assert_eq!(number.to_string(), value.to_string());
        assert_eq!(number.to_u128(), Some(value));
        assert_eq!(number.to_string().parse::<BigNum>().unwrap(), number);
    }
}

#[test]
fn random_u128_values_round_trip() {
    let mut rng = Lcg(0x1234_5678_9ABC_DEF0);

    for _ in 0..1_000 {
        let high = u128::from(rng.next());
        let low = u128::from(rng.next());
        let value = (high << 64) | low;

        let number = BigNum::from(value);
        assert_eq!(number.to_u128(), Some(value));
        assert_eq!(number.to_string(), value.to_string());
        assert_eq!(number.to_string().parse::<BigNum>().unwrap(), number);
    }
}

#[test]
fn conversion_overflow_is_reported() {
    let max: BigNum = u128::MAX.to_string().parse().unwrap();
    assert_eq!(max.to_u128(), Some(u128::MAX));

    let too_big: BigNum = "340282366920938463463374607431768211456".parse().unwrap();
    assert_eq!(too_big.to_u128(), None);

    let too_big_for_u64: BigNum = "18446744073709551616".parse().unwrap();
    assert_eq!(too_big_for_u64.to_u64(), None);
}

#[test]
fn digit_vec_bridge_works() {
    let digits = vec![Digit::Zero, Digit::Four, Digit::Two];
    let number = BigNum::from(digits);
    assert_eq!(number.to_string(), "42");

    let back: Vec<Digit> = number.into();
    assert_eq!(back, [Digit::Four, Digit::Two]);
}
