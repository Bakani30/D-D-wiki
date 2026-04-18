//! Dice engine — parses D&D 5e notation, executes with an injected `Rng`,
//! and returns a `RollResult` that exposes every natural roll so the API
//! layer can show dice to players (per ADR "Dice Visibility").

use std::fmt;
use std::str::FromStr;

use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_COUNT: u32 = 1000;
const MIN_SIDES: u32 = 2;
const MAX_SIDES: u32 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sign {
    Plus,
    Minus,
}

impl Sign {
    fn as_i32(self) -> i32 {
        match self {
            Sign::Plus => 1,
            Sign::Minus => -1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DiceTerm {
    Dice { count: u32, sides: u32, sign: Sign },
    Modifier { value: i32 },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollMode {
    #[default]
    Normal,
    Advantage,
    Disadvantage,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Roll {
    terms: Vec<DiceTerm>,
    mode: RollMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TermResult {
    Dice {
        count: u32,
        sides: u32,
        sign: Sign,
        naturals: Vec<u32>,
        subtotal: i32,
    },
    Modifier {
        value: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollResult {
    pub roll: Roll,
    pub terms: Vec<TermResult>,
    /// The discarded total when mode is Advantage/Disadvantage; `None` otherwise.
    pub alternate_total: Option<i32>,
    pub total: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DiceError {
    #[error("empty dice expression")]
    Empty,
    #[error("unexpected character {0:?}")]
    UnexpectedChar(char),
    #[error("dice count out of range (0..=1000)")]
    InvalidCount,
    #[error("dice sides out of range (2..=1000)")]
    InvalidSides,
    #[error("numeric overflow")]
    Overflow,
    #[error("advantage/disadvantage requires exactly one 1d20 term")]
    AdvantageRequiresD20,
    #[error("missing term after sign")]
    MissingTerm,
}

fn validate_dice(count: u32, sides: u32) -> Result<(), DiceError> {
    if count > MAX_COUNT {
        return Err(DiceError::InvalidCount);
    }
    if !(MIN_SIDES..=MAX_SIDES).contains(&sides) {
        return Err(DiceError::InvalidSides);
    }
    Ok(())
}

impl Roll {
    pub fn new(terms: Vec<DiceTerm>) -> Result<Self, DiceError> {
        if terms.is_empty() {
            return Err(DiceError::Empty);
        }
        for term in &terms {
            if let DiceTerm::Dice { count, sides, .. } = *term {
                validate_dice(count, sides)?;
            }
        }
        Ok(Self {
            terms,
            mode: RollMode::Normal,
        })
    }

    pub fn with_mode(mut self, mode: RollMode) -> Result<Self, DiceError> {
        if !matches!(mode, RollMode::Normal) && !self.is_single_positive_d20() {
            return Err(DiceError::AdvantageRequiresD20);
        }
        self.mode = mode;
        Ok(self)
    }

    fn is_single_positive_d20(&self) -> bool {
        let mut found = false;
        for term in &self.terms {
            if let DiceTerm::Dice { count, sides, sign } = *term {
                if found {
                    return false;
                }
                if count != 1 || sides != 20 || !matches!(sign, Sign::Plus) {
                    return false;
                }
                found = true;
            }
        }
        found
    }

    pub fn terms(&self) -> &[DiceTerm] {
        &self.terms
    }

    pub fn mode(&self) -> RollMode {
        self.mode
    }

    pub fn execute<R: Rng + ?Sized>(&self, rng: &mut R) -> RollResult {
        match self.mode {
            RollMode::Normal => self.execute_once(rng),
            RollMode::Advantage => {
                let a = self.execute_once(rng);
                let b = self.execute_once(rng);
                if a.total >= b.total {
                    RollResult { alternate_total: Some(b.total), ..a }
                } else {
                    RollResult { alternate_total: Some(a.total), ..b }
                }
            }
            RollMode::Disadvantage => {
                let a = self.execute_once(rng);
                let b = self.execute_once(rng);
                if a.total <= b.total {
                    RollResult { alternate_total: Some(b.total), ..a }
                } else {
                    RollResult { alternate_total: Some(a.total), ..b }
                }
            }
        }
    }

    fn execute_once<R: Rng + ?Sized>(&self, rng: &mut R) -> RollResult {
        let mut results = Vec::with_capacity(self.terms.len());
        let mut total: i32 = 0;
        for term in &self.terms {
            match *term {
                DiceTerm::Dice { count, sides, sign } => {
                    let mut naturals = Vec::with_capacity(count as usize);
                    let mut sum: i32 = 0;
                    for _ in 0..count {
                        let n = rng.gen_range(1..=sides);
                        naturals.push(n);
                        sum = sum.saturating_add(n as i32);
                    }
                    let subtotal = sign.as_i32().saturating_mul(sum);
                    total = total.saturating_add(subtotal);
                    results.push(TermResult::Dice {
                        count,
                        sides,
                        sign,
                        naturals,
                        subtotal,
                    });
                }
                DiceTerm::Modifier { value } => {
                    total = total.saturating_add(value);
                    results.push(TermResult::Modifier { value });
                }
            }
        }
        RollResult {
            roll: self.clone(),
            terms: results,
            alternate_total: None,
            total,
        }
    }

    /// Convenience: execute with the thread-local RNG.
    pub fn roll(&self) -> RollResult {
        self.execute(&mut rand::thread_rng())
    }
}

impl fmt::Display for Roll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, term) in self.terms.iter().enumerate() {
            match *term {
                DiceTerm::Dice { count, sides, sign } => {
                    if i == 0 {
                        if matches!(sign, Sign::Minus) {
                            f.write_str("-")?;
                        }
                    } else {
                        f.write_str(if matches!(sign, Sign::Plus) { "+" } else { "-" })?;
                    }
                    write!(f, "{}d{}", count, sides)?;
                }
                DiceTerm::Modifier { value } => {
                    if i == 0 || value < 0 {
                        write!(f, "{}", value)?;
                    } else {
                        write!(f, "+{}", value)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl FromStr for Roll {
    type Err = DiceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s)
    }
}

impl RollResult {
    /// Dice-visible format for narrative splicing: `"1d20+5 → [18]+5 = 23"`.
    /// Adv/Dis mode appends ` (adv)` / ` (dis)` after the expression and
    /// ` [alt N]` after the total.
    pub fn narrative(&self) -> String {
        let mut s = format!("{}", self.roll);
        match self.roll.mode {
            RollMode::Advantage => s.push_str(" (adv)"),
            RollMode::Disadvantage => s.push_str(" (dis)"),
            RollMode::Normal => {}
        }
        s.push_str(" → ");
        for (i, tr) in self.terms.iter().enumerate() {
            match tr {
                TermResult::Dice { sign, naturals, .. } => {
                    if i == 0 {
                        if matches!(sign, Sign::Minus) {
                            s.push('-');
                        }
                    } else {
                        s.push(if matches!(sign, Sign::Plus) { '+' } else { '-' });
                    }
                    s.push('[');
                    for (j, n) in naturals.iter().enumerate() {
                        if j > 0 {
                            s.push(',');
                        }
                        s.push_str(&n.to_string());
                    }
                    s.push(']');
                }
                TermResult::Modifier { value } => {
                    if i == 0 || *value < 0 {
                        s.push_str(&value.to_string());
                    } else {
                        s.push('+');
                        s.push_str(&value.to_string());
                    }
                }
            }
        }
        s.push_str(" = ");
        s.push_str(&self.total.to_string());
        if let Some(alt) = self.alternate_total {
            s.push_str(&format!(" [alt {}]", alt));
        }
        s
    }
}

pub fn parse(s: &str) -> Result<Roll, DiceError> {
    Parser::new(s).parse()
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn parse_u32(&mut self) -> Result<Option<u32>, DiceError> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.bump();
            } else {
                break;
            }
        }
        if start == self.pos {
            return Ok(None);
        }
        let n: u32 = self.input[start..self.pos]
            .parse()
            .map_err(|_| DiceError::Overflow)?;
        Ok(Some(n))
    }

    fn parse(mut self) -> Result<Roll, DiceError> {
        self.skip_ws();
        if self.peek().is_none() {
            return Err(DiceError::Empty);
        }

        let first_sign = match self.peek() {
            Some('+') => {
                self.bump();
                self.skip_ws();
                Sign::Plus
            }
            Some('-') => {
                self.bump();
                self.skip_ws();
                Sign::Minus
            }
            _ => Sign::Plus,
        };

        let mut terms = vec![self.parse_term(first_sign)?];

        loop {
            self.skip_ws();
            match self.peek() {
                Some('+') => {
                    self.bump();
                    self.skip_ws();
                    terms.push(self.parse_term(Sign::Plus)?);
                }
                Some('-') => {
                    self.bump();
                    self.skip_ws();
                    terms.push(self.parse_term(Sign::Minus)?);
                }
                None => break,
                Some(c) => return Err(DiceError::UnexpectedChar(c)),
            }
        }

        Roll::new(terms)
    }

    fn parse_term(&mut self, sign: Sign) -> Result<DiceTerm, DiceError> {
        let count_opt = self.parse_u32()?;
        match self.peek() {
            Some('d') | Some('D') => {
                self.bump();
                let sides = self.parse_u32()?.ok_or(DiceError::InvalidSides)?;
                let count = count_opt.unwrap_or(1);
                validate_dice(count, sides)?;
                Ok(DiceTerm::Dice { count, sides, sign })
            }
            _ => {
                let v = count_opt.ok_or(DiceError::MissingTerm)?;
                let iv = i32::try_from(v).map_err(|_| DiceError::Overflow)?;
                let signed = sign.as_i32().checked_mul(iv).ok_or(DiceError::Overflow)?;
                Ok(DiceTerm::Modifier { value: signed })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn parses_basic_dice() {
        let r = parse("1d20").unwrap();
        assert_eq!(
            r.terms(),
            &[DiceTerm::Dice { count: 1, sides: 20, sign: Sign::Plus }]
        );
    }

    #[test]
    fn parses_with_positive_modifier() {
        let r = parse("1d20+5").unwrap();
        assert_eq!(r.terms().len(), 2);
        assert_eq!(r.terms()[1], DiceTerm::Modifier { value: 5 });
    }

    #[test]
    fn parses_with_negative_modifier() {
        let r = parse("2d6-1").unwrap();
        assert_eq!(r.terms()[1], DiceTerm::Modifier { value: -1 });
    }

    #[test]
    fn parses_compound_damage() {
        let r = parse("1d8+1d6+3").unwrap();
        assert_eq!(r.terms().len(), 3);
        assert_eq!(
            r.terms()[0],
            DiceTerm::Dice { count: 1, sides: 8, sign: Sign::Plus }
        );
        assert_eq!(
            r.terms()[1],
            DiceTerm::Dice { count: 1, sides: 6, sign: Sign::Plus }
        );
        assert_eq!(r.terms()[2], DiceTerm::Modifier { value: 3 });
    }

    #[test]
    fn parses_implicit_count() {
        let r = parse("d20").unwrap();
        assert_eq!(
            r.terms(),
            &[DiceTerm::Dice { count: 1, sides: 20, sign: Sign::Plus }]
        );
    }

    #[test]
    fn parses_uppercase_d() {
        let r = parse("1D20").unwrap();
        assert_eq!(
            r.terms()[0],
            DiceTerm::Dice { count: 1, sides: 20, sign: Sign::Plus }
        );
    }

    #[test]
    fn parses_whitespace() {
        let r = parse(" 1d20 + 5 ").unwrap();
        assert_eq!(r.terms().len(), 2);
    }

    #[test]
    fn zero_dice_yields_modifier_total() {
        let r = parse("0d6+7").unwrap();
        let mut rng = StdRng::seed_from_u64(0);
        assert_eq!(r.execute(&mut rng).total, 7);
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(parse(""), Err(DiceError::Empty));
        assert_eq!(parse("   "), Err(DiceError::Empty));
    }

    #[test]
    fn rejects_missing_sides() {
        assert_eq!(parse("1d"), Err(DiceError::InvalidSides));
    }

    #[test]
    fn rejects_invalid_sides() {
        assert_eq!(parse("1d1"), Err(DiceError::InvalidSides));
        assert_eq!(parse("1d1001"), Err(DiceError::InvalidSides));
    }

    #[test]
    fn rejects_too_many_dice() {
        assert_eq!(parse("1001d20"), Err(DiceError::InvalidCount));
    }

    #[test]
    fn rejects_trailing_sign() {
        assert_eq!(parse("1d20+"), Err(DiceError::MissingTerm));
    }

    #[test]
    fn rejects_garbage() {
        assert!(matches!(parse("1x20"), Err(DiceError::UnexpectedChar('x'))));
    }

    #[test]
    fn rejects_parser_overflow() {
        assert_eq!(parse("9999999999d20"), Err(DiceError::Overflow));
    }

    #[test]
    fn advantage_accepts_1d20_with_modifier() {
        let r = Roll::new(vec![
            DiceTerm::Dice { count: 1, sides: 20, sign: Sign::Plus },
            DiceTerm::Modifier { value: 5 },
        ])
        .unwrap()
        .with_mode(RollMode::Advantage)
        .unwrap();
        assert_eq!(r.mode(), RollMode::Advantage);
    }

    #[test]
    fn advantage_rejects_damage_expression() {
        let err = Roll::new(vec![
            DiceTerm::Dice { count: 1, sides: 8, sign: Sign::Plus },
            DiceTerm::Modifier { value: 3 },
        ])
        .unwrap()
        .with_mode(RollMode::Advantage)
        .unwrap_err();
        assert_eq!(err, DiceError::AdvantageRequiresD20);
    }

    #[test]
    fn advantage_rejects_multiple_d20() {
        let err = Roll::new(vec![DiceTerm::Dice {
            count: 2,
            sides: 20,
            sign: Sign::Plus,
        }])
        .unwrap()
        .with_mode(RollMode::Advantage)
        .unwrap_err();
        assert_eq!(err, DiceError::AdvantageRequiresD20);
    }

    #[test]
    fn narrative_exposes_naturals() {
        let roll = parse("1d20+5").unwrap();
        let mut rng = StdRng::seed_from_u64(42);
        let r = roll.execute(&mut rng);
        let n = r.narrative();
        assert!(n.contains("1d20+5"));
        assert!(n.contains('[') && n.contains(']'));
        assert!(n.contains(&format!("= {}", r.total)));
    }

    #[test]
    fn narrative_shows_alternate_for_advantage() {
        let roll = parse("1d20+5")
            .unwrap()
            .with_mode(RollMode::Advantage)
            .unwrap();
        let mut rng = StdRng::seed_from_u64(42);
        let r = roll.execute(&mut rng);
        assert!(r.alternate_total.is_some());
        assert!(r.narrative().contains("(adv)"));
        assert!(r.narrative().contains("alt"));
    }

    proptest! {
        #[test]
        fn parser_never_panics(s in "[0-9dD +\\-]{0,32}") {
            let _ = parse(&s);
        }

        #[test]
        fn parser_tolerates_printable_ascii(s in "[ -~]{0,16}") {
            let _ = parse(&s);
        }

        #[test]
        fn single_term_bounds(
            count in 0u32..=20,
            sides in 2u32..=100,
            modifier in -1000i32..=1000,
            seed in any::<u64>(),
        ) {
            let roll = Roll::new(vec![
                DiceTerm::Dice { count, sides, sign: Sign::Plus },
                DiceTerm::Modifier { value: modifier },
            ]).unwrap();
            let mut rng = StdRng::seed_from_u64(seed);
            let r = roll.execute(&mut rng);
            let min = count as i32 + modifier;
            let max = count as i32 * sides as i32 + modifier;
            prop_assert!(r.total >= min, "total {} below min {}", r.total, min);
            prop_assert!(r.total <= max, "total {} above max {}", r.total, max);
        }

        #[test]
        fn compound_bounds(
            count_a in 0u32..=10,
            sides_a in 2u32..=20,
            count_b in 0u32..=10,
            sides_b in 2u32..=20,
            modifier in -100i32..=100,
            seed in any::<u64>(),
        ) {
            let roll = Roll::new(vec![
                DiceTerm::Dice { count: count_a, sides: sides_a, sign: Sign::Plus },
                DiceTerm::Dice { count: count_b, sides: sides_b, sign: Sign::Plus },
                DiceTerm::Modifier { value: modifier },
            ]).unwrap();
            let mut rng = StdRng::seed_from_u64(seed);
            let r = roll.execute(&mut rng);
            let min = count_a as i32 + count_b as i32 + modifier;
            let max = count_a as i32 * sides_a as i32
                + count_b as i32 * sides_b as i32
                + modifier;
            prop_assert!(r.total >= min);
            prop_assert!(r.total <= max);
        }

        #[test]
        fn naturals_always_in_die_range(
            count in 0u32..=20,
            sides in 2u32..=100,
            seed in any::<u64>(),
        ) {
            let roll = Roll::new(vec![
                DiceTerm::Dice { count, sides, sign: Sign::Plus },
            ]).unwrap();
            let mut rng = StdRng::seed_from_u64(seed);
            let r = roll.execute(&mut rng);
            if let TermResult::Dice { naturals, .. } = &r.terms[0] {
                prop_assert_eq!(naturals.len() as u32, count);
                for n in naturals {
                    prop_assert!(*n >= 1 && *n <= sides);
                }
            }
        }

        #[test]
        fn display_roundtrips(
            count in 0u32..=20,
            sides in 2u32..=100,
            modifier in -1000i32..=1000,
        ) {
            let original = Roll::new(vec![
                DiceTerm::Dice { count, sides, sign: Sign::Plus },
                DiceTerm::Modifier { value: modifier },
            ]).unwrap();
            let s = format!("{}", original);
            let reparsed = parse(&s).unwrap();
            prop_assert_eq!(original.terms(), reparsed.terms());
        }

        #[test]
        fn advantage_picks_max(seed in any::<u64>()) {
            let roll = Roll::new(vec![
                DiceTerm::Dice { count: 1, sides: 20, sign: Sign::Plus },
                DiceTerm::Modifier { value: 5 },
            ]).unwrap().with_mode(RollMode::Advantage).unwrap();
            let mut rng = StdRng::seed_from_u64(seed);
            let r = roll.execute(&mut rng);
            let alt = r.alternate_total.expect("advantage records alt");
            prop_assert!(r.total >= alt);
        }

        #[test]
        fn disadvantage_picks_min(seed in any::<u64>()) {
            let roll = Roll::new(vec![
                DiceTerm::Dice { count: 1, sides: 20, sign: Sign::Plus },
                DiceTerm::Modifier { value: 5 },
            ]).unwrap().with_mode(RollMode::Disadvantage).unwrap();
            let mut rng = StdRng::seed_from_u64(seed);
            let r = roll.execute(&mut rng);
            let alt = r.alternate_total.expect("disadvantage records alt");
            prop_assert!(r.total <= alt);
        }

        #[test]
        fn zero_count_equals_modifier(
            modifier in -1000i32..=1000,
            sides in 2u32..=100,
            seed in any::<u64>(),
        ) {
            let roll = Roll::new(vec![
                DiceTerm::Dice { count: 0, sides, sign: Sign::Plus },
                DiceTerm::Modifier { value: modifier },
            ]).unwrap();
            let mut rng = StdRng::seed_from_u64(seed);
            prop_assert_eq!(roll.execute(&mut rng).total, modifier);
        }

        #[test]
        fn modifier_sign_symmetry(
            count in 1u32..=10,
            sides in 2u32..=20,
            modifier in 0i32..=100,
            seed in any::<u64>(),
        ) {
            let minus = Roll::new(vec![
                DiceTerm::Dice { count, sides, sign: Sign::Plus },
                DiceTerm::Modifier { value: -modifier },
            ]).unwrap();
            let plus = Roll::new(vec![
                DiceTerm::Dice { count, sides, sign: Sign::Plus },
                DiceTerm::Modifier { value: modifier },
            ]).unwrap();
            let mut rng1 = StdRng::seed_from_u64(seed);
            let mut rng2 = StdRng::seed_from_u64(seed);
            let r1 = minus.execute(&mut rng1);
            let r2 = plus.execute(&mut rng2);
            prop_assert_eq!(r1.total + 2 * modifier, r2.total);
        }
    }
}
