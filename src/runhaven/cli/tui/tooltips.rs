const ROTATE_EVERY_TICKS: u64 = 8;

const TIPS: &[&str] = &[
    "RunHaven shows the exact CLI command before launch.",
    "Secure runs do not mount your home or credential folders.",
    "Press p to hide or show Cubby.",
    "Enter inspects an agent before you launch it.",
    "Blocked network attempts are shown in the run ledger.",
];

pub(super) fn tip_for_tick(ticks: u64) -> &'static str {
    let index = (ticks / ROTATE_EVERY_TICKS) as usize % TIPS.len();
    TIPS[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tips_rotate_on_fixed_tick_windows() {
        assert_eq!(tip_for_tick(0), TIPS[0]);
        assert_eq!(tip_for_tick(ROTATE_EVERY_TICKS - 1), TIPS[0]);
        assert_eq!(tip_for_tick(ROTATE_EVERY_TICKS), TIPS[1]);
    }

    #[test]
    fn tips_wrap_without_empty_entries() {
        for tick in 0..(ROTATE_EVERY_TICKS * TIPS.len() as u64 * 2) {
            assert!(!tip_for_tick(tick).is_empty());
        }
    }
}
