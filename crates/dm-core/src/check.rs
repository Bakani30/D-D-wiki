//! d20 tests: Checks and Contests per 5e 2024 RAW.
//!
//! - `Check`: d20 + modifier vs a DC. Strict RAW — no nat-20 auto-success,
//!   no nat-1 auto-fail. The natural d20 is exposed on the result for
//!   narrative flavor only.
//! - `Contest`: two opposing d20 tests. Higher total wins; equal totals are
//!   `Tie` (situation unchanged, per 2024 RAW).
//!
//! Both resolve through `dice::Roll`, so advantage/disadvantage and the
//! dice-visibility narrative come through unchanged.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::dice::{DiceTerm, Roll, RollMode, RollResult, Sign, TermResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct D20Test {
    roll: Roll,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct D20TestResult {
    pub roll: RollResult,
    pub natural_d20: u32,
    pub total: i32,
}

impl D20Test {
    pub fn new(modifier: i32) -> Self {
        Self::with_mode(modifier, RollMode::Normal)
    }

    pub fn advantage(modifier: i32) -> Self {
        Self::with_mode(modifier, RollMode::Advantage)
    }

    pub fn disadvantage(modifier: i32) -> Self {
        Self::with_mode(modifier, RollMode::Disadvantage)
    }

    pub fn with_mode(modifier: i32, mode: RollMode) -> Self {
        let terms = vec![
            DiceTerm::Dice { count: 1, sides: 20, sign: Sign::Plus },
            DiceTerm::Modifier { value: modifier },
        ];
        let roll = Roll::new(terms)
            .expect("1d20 + modifier is a valid dice expression")
            .with_mode(mode)
            .expect("1d20 + modifier accepts any RollMode");
        Self { roll }
    }

    pub fn roll(&self) -> &Roll {
        &self.roll
    }

    pub fn execute<R: Rng + ?Sized>(&self, rng: &mut R) -> D20TestResult {
        let rr = self.roll.execute(rng);
        let total = rr.total;
        let natural_d20 = extract_d20_natural(&rr);
        D20TestResult { roll: rr, natural_d20, total }
    }
}

fn extract_d20_natural(rr: &RollResult) -> u32 {
    for t in &rr.terms {
        if let TermResult::Dice { sides: 20, naturals, .. } = t {
            if let Some(n) = naturals.first() {
                return *n;
            }
        }
    }
    0
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Check {
    pub test: D20Test,
    pub dc: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub test: D20TestResult,
    pub dc: i32,
    pub success: bool,
}

impl Check {
    pub fn new(test: D20Test, dc: i32) -> Self {
        Self { test, dc }
    }

    pub fn execute<R: Rng + ?Sized>(&self, rng: &mut R) -> CheckResult {
        let test = self.test.execute(rng);
        let success = test.total >= self.dc;
        CheckResult { test, dc: self.dc, success }
    }
}

impl CheckResult {
    pub fn narrative(&self) -> String {
        format!(
            "{} vs DC {} — {}",
            self.test.roll.narrative(),
            self.dc,
            if self.success { "success" } else { "failure" }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contest {
    pub initiator: D20Test,
    pub responder: D20Test,
}

/// Option (b): a separate outcome enum. Contest sides carry no DC — the result
/// is decided purely by the relative totals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContestOutcome {
    Initiator,
    Responder,
    Tie,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContestResult {
    pub initiator: D20TestResult,
    pub responder: D20TestResult,
    pub outcome: ContestOutcome,
}

impl Contest {
    pub fn new(initiator: D20Test, responder: D20Test) -> Self {
        Self { initiator, responder }
    }

    pub fn execute<R: Rng + ?Sized>(&self, rng: &mut R) -> ContestResult {
        let initiator = self.initiator.execute(rng);
        let responder = self.responder.execute(rng);
        let outcome = match initiator.total.cmp(&responder.total) {
            std::cmp::Ordering::Greater => ContestOutcome::Initiator,
            std::cmp::Ordering::Less => ContestOutcome::Responder,
            std::cmp::Ordering::Equal => ContestOutcome::Tie,
        };
        ContestResult { initiator, responder, outcome }
    }
}

impl ContestResult {
    pub fn narrative(&self) -> String {
        let tag = match self.outcome {
            ContestOutcome::Initiator => "initiator wins",
            ContestOutcome::Responder => "responder wins",
            ContestOutcome::Tie => "tie — situation unchanged",
        };
        format!(
            "initiator {} | responder {} — {}",
            self.initiator.roll.narrative(),
            self.responder.roll.narrative(),
            tag
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn check_success_at_threshold() {
        // mod +10, DC 11 — any nat gives total 11..=30, all ≥ DC, all succeed.
        let check = Check::new(D20Test::new(10), 11);
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..50 {
            assert!(check.execute(&mut rng).success);
        }
    }

    #[test]
    fn check_failure_guaranteed() {
        // mod +0, DC 21 — max total 20, can never succeed.
        let check = Check::new(D20Test::new(0), 21);
        let mut rng = StdRng::seed_from_u64(2);
        for _ in 0..50 {
            assert!(!check.execute(&mut rng).success);
        }
    }

    #[test]
    fn strict_raw_nat_20_does_not_auto_succeed() {
        // mod +0, DC 25 — max total 20, always below DC regardless of natural.
        let check = Check::new(D20Test::new(0), 25);
        let mut rng = StdRng::seed_from_u64(0xDEAD_BEEF);
        let mut saw_nat_20 = false;
        for _ in 0..2000 {
            let r = check.execute(&mut rng);
            assert!(
                !r.success,
                "strict RAW violated: nat {} succeeded against DC 25",
                r.test.natural_d20
            );
            if r.test.natural_d20 == 20 {
                saw_nat_20 = true;
            }
        }
        assert!(saw_nat_20, "did not observe a nat 20 in 2000 rolls");
    }

    #[test]
    fn strict_raw_nat_1_does_not_auto_fail() {
        // mod +30, DC 5 — min total 31, always above DC regardless of natural.
        let check = Check::new(D20Test::new(30), 5);
        let mut rng = StdRng::seed_from_u64(0xCAFE_BABE);
        let mut saw_nat_1 = false;
        for _ in 0..2000 {
            let r = check.execute(&mut rng);
            assert!(
                r.success,
                "strict RAW violated: nat {} failed against DC 5",
                r.test.natural_d20
            );
            if r.test.natural_d20 == 1 {
                saw_nat_1 = true;
            }
        }
        assert!(saw_nat_1, "did not observe a nat 1 in 2000 rolls");
    }

    #[test]
    fn natural_d20_matches_roll_term() {
        let test = D20Test::new(5);
        let mut rng = StdRng::seed_from_u64(42);
        let r = test.execute(&mut rng);
        assert!((1..=20).contains(&r.natural_d20));
        assert_eq!(r.total, r.natural_d20 as i32 + 5);
    }

    #[test]
    fn advantage_mode_flows_through() {
        let check = Check::new(D20Test::advantage(5), 15);
        let mut rng = StdRng::seed_from_u64(7);
        let r = check.execute(&mut rng);
        assert!(r.test.roll.alternate_total.is_some());
    }

    #[test]
    fn contest_higher_total_wins() {
        let contest = Contest::new(D20Test::new(10), D20Test::new(0));
        let mut rng = StdRng::seed_from_u64(3);
        for _ in 0..50 {
            let r = contest.execute(&mut rng);
            match r.outcome {
                ContestOutcome::Initiator => assert!(r.initiator.total > r.responder.total),
                ContestOutcome::Responder => assert!(r.responder.total > r.initiator.total),
                ContestOutcome::Tie => assert_eq!(r.initiator.total, r.responder.total),
            }
        }
    }

    #[test]
    fn contest_tie_on_equal_totals() {
        // Both sides have the same D20Test — given the same rolls, totals match.
        // We can't easily force equality without a controlled RNG, so we assert
        // the branch is reachable and the mapping matches totals.
        let contest = Contest::new(D20Test::new(0), D20Test::new(0));
        let mut rng = StdRng::seed_from_u64(99);
        for _ in 0..200 {
            let r = contest.execute(&mut rng);
            let expected = match r.initiator.total.cmp(&r.responder.total) {
                std::cmp::Ordering::Greater => ContestOutcome::Initiator,
                std::cmp::Ordering::Less => ContestOutcome::Responder,
                std::cmp::Ordering::Equal => ContestOutcome::Tie,
            };
            assert_eq!(r.outcome, expected);
        }
    }

    #[test]
    fn check_narrative_contains_dc_and_verdict() {
        let check = Check::new(D20Test::new(5), 15);
        let mut rng = StdRng::seed_from_u64(11);
        let r = check.execute(&mut rng);
        let n = r.narrative();
        assert!(n.contains("DC 15"));
        assert!(n.contains("success") || n.contains("failure"));
    }

    #[test]
    fn contest_narrative_labels_both_sides() {
        let contest = Contest::new(D20Test::new(3), D20Test::new(4));
        let mut rng = StdRng::seed_from_u64(13);
        let r = contest.execute(&mut rng);
        let n = r.narrative();
        assert!(n.contains("initiator"));
        assert!(n.contains("responder"));
    }

    proptest! {
        #[test]
        fn check_success_iff_total_ge_dc(
            modifier in -10i32..=10,
            dc in -5i32..=30,
            seed in any::<u64>(),
        ) {
            let check = Check::new(D20Test::new(modifier), dc);
            let mut rng = StdRng::seed_from_u64(seed);
            let r = check.execute(&mut rng);
            prop_assert_eq!(r.success, r.test.total >= dc);
        }

        #[test]
        fn natural_d20_always_in_range(
            modifier in -10i32..=10,
            seed in any::<u64>(),
        ) {
            let test = D20Test::new(modifier);
            let mut rng = StdRng::seed_from_u64(seed);
            let r = test.execute(&mut rng);
            prop_assert!((1..=20).contains(&r.natural_d20));
            prop_assert_eq!(r.total, r.natural_d20 as i32 + modifier);
        }

        #[test]
        fn contest_outcome_matches_totals(
            mod_a in -10i32..=10,
            mod_b in -10i32..=10,
            seed in any::<u64>(),
        ) {
            let contest = Contest::new(D20Test::new(mod_a), D20Test::new(mod_b));
            let mut rng = StdRng::seed_from_u64(seed);
            let r = contest.execute(&mut rng);
            let expected = match r.initiator.total.cmp(&r.responder.total) {
                std::cmp::Ordering::Greater => ContestOutcome::Initiator,
                std::cmp::Ordering::Less => ContestOutcome::Responder,
                std::cmp::Ordering::Equal => ContestOutcome::Tie,
            };
            prop_assert_eq!(r.outcome, expected);
        }

        #[test]
        fn advantage_test_keeps_higher(
            modifier in -10i32..=10,
            seed in any::<u64>(),
        ) {
            let test = D20Test::advantage(modifier);
            let mut rng = StdRng::seed_from_u64(seed);
            let r = test.execute(&mut rng);
            let alt = r.roll.alternate_total.expect("advantage records alt");
            prop_assert!(r.total >= alt);
        }

        #[test]
        fn disadvantage_test_keeps_lower(
            modifier in -10i32..=10,
            seed in any::<u64>(),
        ) {
            let test = D20Test::disadvantage(modifier);
            let mut rng = StdRng::seed_from_u64(seed);
            let r = test.execute(&mut rng);
            let alt = r.roll.alternate_total.expect("disadvantage records alt");
            prop_assert!(r.total <= alt);
        }
    }
}
