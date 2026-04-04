use chrono::TimeZone;
use cron::Schedule;
use std::str::FromStr;
fn main() {
    let tz: chrono_tz::Tz = "America/Phoenix".parse().unwrap();
    let now_utc = chrono::Utc.with_ymd_and_hms(2026, 4, 3, 8, 56, 38).unwrap();
    let now_local = now_utc.with_timezone(&tz);
    println!("now_utc={now_utc} now_local={now_local}");
    for (name, expr) in [
        ("aws-security", "0 0 * * * * *"),
        ("weekday-cal", "0 30 7 * * 1-5 *"),
        ("wilken", "0 0 7-21 * * 1-5 *"),
        ("post-meeting", "0 0 10,12,14,16,18 * * 1-5 *"),
    ] {
        let s = Schedule::from_str(expr).unwrap();
        if let Some(n) = s.after(&now_local).next() {
            let nu = n.with_timezone(&chrono::Utc);
            let d = nu - now_utc;
            println!(
                "{name:30} next={} delay={}s",
                nu.format("%H:%M UTC"),
                d.num_seconds()
            );
        } else {
            println!("{name:30} NONE!");
        }
    }
}
