use super::{FacilityPower, MAX_OUTAGE_DELAY_SECS, MIN_OUTAGE_DELAY_SECS};

#[test]
fn seeded_outages_repeat_after_each_repair() {
    let mut power = FacilityPower::new(42);
    assert_eq!(power, FacilityPower::new(42));
    assert!((MIN_OUTAGE_DELAY_SECS..MAX_OUTAGE_DELAY_SECS).contains(&power.remaining_secs));
    power.tick(power.remaining_secs - 0.1);
    assert!(power.on);
    power.tick(0.2);
    assert!(!power.on);
    assert!(!power.outage_pending);
    power.restore();
    assert!(power.on);
    assert!(power.outage_pending);
    assert!((MIN_OUTAGE_DELAY_SECS..MAX_OUTAGE_DELAY_SECS).contains(&power.remaining_secs));
    assert_eq!(power, {
        let mut another = FacilityPower::new(42);
        another.tick(1000.0);
        another.restore();
        another
    });
    power.tick(power.remaining_secs);
    assert!(!power.on);
    power.restore();
    assert!(power.on);
    assert!(power.outage_pending);
    power.tick(power.remaining_secs);
    assert!(!power.on);
}
