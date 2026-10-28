//! Chooses what the matrix shows: a test the driver asked for, else the first matching rule.

use std::collections::VecDeque;

use crate::pack::{Animation, Hold, Pack};
use crate::signal::Signal;

/// Not an animation name, so it falls back to the blank animation.
pub const OFF: &str = "off";

#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    /// The 1-based rule in mapping.json.
    Rule(usize),
    Test,
    NoSim,
}

impl Source {
    pub fn describe(&self) -> String {
        match self {
            Source::Rule(n) => format!("rule {n} of mapping.json"),
            Source::Test => "test".into(),
            Source::NoSim => "no sim: matrix cleared".into(),
        }
    }
}

#[derive(Debug)]
struct Test {
    name: String,
    custom: Option<Animation>,
    until: u64,
}

#[derive(Debug, Default)]
pub struct Player {
    showing: Option<(String, bool, u64)>,
    rule_key: Option<String>,
    rule_since: u64,
    test: Option<Test>,
    queue: VecDeque<String>,
    each_ms: u64,
}

pub struct Choice<'a> {
    pub name: String,
    pub source: Source,
    pub anim: &'a Animation,
    /// When this animation started, for `Animation::frame_at`.
    pub started: u64,
}

impl Player {
    /// Plays each animation in turn for `each_ms`, ahead of the flag.
    pub fn play_tests(&mut self, names: Vec<String>, now: u64, each_ms: u64) {
        let mut q: VecDeque<String> = names.into();
        let Some(first) = q.pop_front() else { return };
        self.queue = q;
        self.each_ms = each_ms;
        self.test = Some(Test { name: first, custom: None, until: now + each_ms });
    }

    /// Plays an animation that is not in the pack (the editor's working copy).
    pub fn play_custom(&mut self, anim: Animation, now: u64, ms: u64) {
        self.queue.clear();
        self.test = Some(Test { name: format!("{} (editing)", anim.id), custom: Some(anim), until: now + ms });
    }

    pub fn testing(&self) -> bool {
        self.test.is_some()
    }

    pub fn choose<'a>(
        &'a mut self,
        now: u64,
        signal: Option<&Signal>,
        pack: &'a Pack,
        green_hold_s: f64,
        blank: &'a Animation,
    ) -> Choice<'a> {
        if self.test.as_ref().is_some_and(|t| now >= t.until) {
            self.test = self.queue.pop_front().map(|name| Test { name, custom: None, until: now + self.each_ms });
        }
        let (name, source, custom) = if let Some(t) = &self.test {
            (t.name.clone(), Source::Test, t.custom.is_some())
        } else if let Some(sig) = signal {
            self.by_rule(now, sig, pack, green_hold_s)
        } else {
            self.rule_key = None;
            (OFF.to_string(), Source::NoSim, false)
        };
        let fresh = !matches!(&self.showing, Some((n, c, _)) if *n == name && *c == custom);
        if fresh {
            self.showing = Some((name.clone(), custom, now));
        }
        let started = self.showing.as_ref().map_or(now, |s| s.2);
        let anim = match &self.test {
            Some(Test { custom: Some(a), .. }) if source == Source::Test => a,
            _ => pack.get(&name).unwrap_or(blank),
        };
        Choice { name, source, anim, started }
    }

    fn by_rule(&mut self, now: u64, sig: &Signal, pack: &Pack, green_hold_s: f64) -> (String, Source, bool) {
        let Some((i, rule)) = pack.mapping.iter().enumerate().find(|(_, r)| r.when.matches(sig)) else {
            self.rule_key = None;
            return ("idle".into(), Source::NoSim, false);
        };
        let key = if rule.hold.is_some() { format!("hold:{}", rule.show) } else { format!("rule:{i}") };
        if self.rule_key.as_deref() != Some(&key) {
            self.rule_key = Some(key);
            self.rule_since = now;
        }
        let hold_s = match rule.hold {
            Some(Hold::Seconds(s)) => Some(s),
            Some(Hold::GreenHoldSetting) => Some(green_hold_s),
            None => None,
        };
        let expired = hold_s.is_some_and(|s| now.saturating_sub(self.rule_since) as f64 > s * 1000.0);
        let name = if expired { rule.then.clone().unwrap_or_else(|| "idle".into()) } else { rule.show.clone() };
        (name, Source::Rule(i + 1), false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::tests::classic;
    use crate::signal::{DriverFlag, Flag, StartLight};

    fn sig(primary: Option<Flag>) -> Signal {
        Signal { primary, ..Signal::default() }
    }

    fn pick(p: &mut Player, now: u64, s: Option<&Signal>, pack: &Pack) -> (String, Source) {
        let blank = Animation::blank();
        let c = p.choose(now, s, pack, 3.0, &blank);
        (c.name, c.source)
    }

    #[test]
    fn the_driver_flags_win_over_the_session_flag() {
        let pack = classic();
        let mut p = Player::default();
        let mut s = sig(Some(Flag::Yellow));
        s.waving = true;
        s.driver = vec![DriverFlag::Repair];
        assert_eq!(pick(&mut p, 0, Some(&s), &pack).0, "meatball");
        s.driver.push(DriverFlag::Disqualify);
        assert_eq!(pick(&mut p, 10, Some(&s), &pack).0, "black-x");
    }

    #[test]
    fn a_waving_yellow_and_a_local_yellow_look_different() {
        let pack = classic();
        let mut p = Player::default();
        let mut s = sig(Some(Flag::Yellow));
        assert_eq!(pick(&mut p, 0, Some(&s), &pack).0, "yellow");
        s.waving = true;
        assert_eq!(pick(&mut p, 10, Some(&s), &pack).0, "yellow-waving");
    }

    #[test]
    fn green_flashes_for_the_hold_and_then_the_matrix_goes_idle() {
        let pack = classic();
        let mut p = Player::default();
        let g = sig(Some(Flag::Green));
        assert_eq!(pick(&mut p, 1_000, Some(&g), &pack).0, "green-flash");
        assert_eq!(pick(&mut p, 3_900, Some(&g), &pack).0, "green-flash");
        assert_eq!(pick(&mut p, 4_100, Some(&g), &pack).0, "idle");
    }

    #[test]
    fn go_then_green_is_one_hold_not_two() {
        let pack = classic();
        let mut p = Player::default();
        let mut go = sig(Some(Flag::Green));
        go.start = Some(StartLight::Go);
        assert_eq!(pick(&mut p, 0, Some(&go), &pack).0, "green-flash");
        let green = sig(Some(Flag::Green));
        assert_eq!(pick(&mut p, 2_000, Some(&green), &pack).0, "green-flash");
        assert_eq!(pick(&mut p, 3_100, Some(&green), &pack).0, "idle");
    }

    #[test]
    fn no_sim_clears_the_matrix_whatever_the_pack_says() {
        let pack = classic();
        let mut p = Player::default();
        assert_eq!(pick(&mut p, 0, None, &pack), (OFF.into(), Source::NoSim));
    }

    #[test]
    fn tests_play_in_order_then_hand_back_to_the_flag() {
        let pack = classic();
        let mut p = Player::default();
        let s = sig(Some(Flag::Blue));
        p.play_tests(vec!["red".into(), "white".into()], 0, 1_000);
        assert_eq!(pick(&mut p, 10, Some(&s), &pack), ("red".into(), Source::Test));
        assert_eq!(pick(&mut p, 1_010, Some(&s), &pack), ("white".into(), Source::Test));
        assert_eq!(pick(&mut p, 2_100, Some(&s), &pack).0, "blue-flash");
    }

    #[test]
    fn an_animation_restarts_only_when_it_changes() {
        let pack = classic();
        let mut p = Player::default();
        let blank = Animation::blank();
        let s = sig(Some(Flag::Blue));
        assert_eq!(p.choose(100, Some(&s), &pack, 3.0, &blank).started, 100);
        assert_eq!(p.choose(900, Some(&s), &pack, 3.0, &blank).started, 100);
        let w = sig(Some(Flag::White));
        assert_eq!(p.choose(1_000, Some(&w), &pack, 3.0, &blank).started, 1_000);
    }
}
