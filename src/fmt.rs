#[macro_export]
macro_rules! println_stats {
    ($label:expr, $value:expr, $unit:expr) => {
        println!("{:<36} {:>10.2} {}", $label, $value, $unit);
    };
}

#[macro_export]
macro_rules! println_metric {
    ($label:expr, $value:expr) => {
        let (value, unit) = ($value.as_value(), $value.as_unit());
        println_stats!($label, value, unit);
    };
}

#[macro_export]
macro_rules! println_time_ms {
    ($label:expr, $value:expr) => {
        println!(
            "{:<36} {}",
            $label,
            Duration::from_millis($value as u64).as_human_readable()
        );
    };
}

#[macro_export]
macro_rules! println_duration {
    ($label:expr, $value:expr) => {
        println!("{:<36} {}", $label, $value.as_human_readable());
    };
}

#[macro_export]
macro_rules! shout {
    ($label:expr) => {
        let standard_font = FIGfont::standard().unwrap();
        let figure = standard_font.convert($label);
        assert!(figure.is_some());
        println!("{}", figure.unwrap());
    };
}
