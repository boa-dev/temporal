#[diplomat::bridge]
#[diplomat::abi_rename = "temporal_rs_{0}"]
#[diplomat::attr(auto, namespace = "temporal_rs")]
pub mod ffi {
    use temporal_rs::options;

    #[diplomat::enum_convert(options::ArithmeticOverflow)]
    pub enum ArithmeticOverflow {
        Constrain,
        Reject,
    }
    #[diplomat::enum_convert(options::Disambiguation)]
    pub enum Disambiguation {
        Compatible,
        Earlier,
        Later,
        Reject,
    }

    #[diplomat::enum_convert(options::DisplayCalendar)]
    pub enum DisplayCalendar {
        Auto,
        Always,
        Never,
        Critical,
    }

    #[diplomat::enum_convert(options::DisplayOffset)]
    pub enum DisplayOffset {
        Auto,
        Never,
    }

    #[diplomat::enum_convert(options::DisplayTimeZone)]
    pub enum DisplayTimeZone {
        Auto,
        Never,
        Critical,
    }

    #[diplomat::enum_convert(options::DurationOverflow)]
    pub enum DurationOverflow {
        Constrain,
        Balance,
    }

    #[diplomat::enum_convert(options::OffsetDisambiguation)]
    pub enum OffsetDisambiguation {
        Use,
        Prefer,
        Ignore,
        Reject,
    }

    #[diplomat::enum_convert(options::TemporalRoundingMode)]
    pub enum TemporalRoundingMode {
        Ceil,
        Floor,
        Expand,
        Trunc,
        HalfCeil,
        HalfFloor,
        HalfExpand,
        HalfTrunc,
        HalfEven,
    }

    #[diplomat::enum_convert(options::TemporalUnit)]
    pub enum TemporalUnit {
        Auto = 0,
        Nanosecond = 1,
        Microsecond = 2,
        Millisecond = 3,
        Second = 4,
        Minute = 5,
        Hour = 6,
        Day = 7,
        Week = 8,
        Month = 9,
        Year = 10,
    }

    #[diplomat::enum_convert(options::TemporalUnsignedRoundingMode)]
    pub enum TemporalUnsignedRoundingMode {
        Infinity,
        Zero,
        HalfInfinity,
        HalfZero,
        HalfEven,
    }
}
