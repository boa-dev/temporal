#[diplomat::bridge]
#[diplomat::abi_rename = "temporal_rs_{0}"]
#[diplomat::attr(auto, namespace = "temporal_rs")]
pub mod ffi {
    use crate::error::ffi::TemporalError;
    use diplomat_runtime::DiplomatStr;

    #[diplomat::enum_convert(icu_calendar::any_calendar::AnyCalendarKind, needs_wildcard)]
    pub enum AnyCalendarKind {
        Buddhist,
        Chinese,
        Coptic,
        Dangi,
        Ethiopian,
        EthiopianAmeteAlem,
        Gregorian,
        Hebrew,
        Indian,
        IslamicCivil,
        IslamicObservational,
        IslamicTabular,
        IslamicUmmAlQura,
        Iso,
        Japanese,
        JapaneseExtended,
        Persian,
        Roc,
    }

    impl AnyCalendarKind {
        pub fn get_for_bcp47_string(s: &DiplomatStr) -> Option<Self> {
            icu_calendar::any_calendar::AnyCalendarKind::get_for_bcp47_bytes(s).map(Into::into)
        }
    }

    #[diplomat::opaque]
    #[diplomat::transparent_convert]
    pub struct Calendar(pub temporal_rs::Calendar);

    impl Calendar {
        pub fn create(kind: AnyCalendarKind) -> Box<Self> {
            Box::new(Calendar(temporal_rs::Calendar::new(kind.into())))
        }

        pub fn from_utf8(s: &DiplomatStr) -> Result<Box<Self>, TemporalError> {
            temporal_rs::Calendar::from_utf8(s)
                .map(|c| Box::new(Calendar(c)))
                .map_err(Into::into)
        }

        pub fn is_iso(&self) -> bool {
            self.0.is_iso()
        }

        pub fn identifier(&self) -> &'static str {
            self.0.identifier()
        }

        // TODO the rest of calendar (needs all the date/time types)
    }
}
