/// # Date Equations
///
/// Date Equations is a library focused on implementing
/// small, highly performant calendar calculations. Currently,
/// the implementation is informed by the work done by
/// Cassio Neri and Lorenz Schneider on applying Euclidean
/// affine functions to calendar algorithms.
///
/// ``` rust
/// use date_equations::gregorian;
///
/// let date = gregorian::ymd_from_epoch_days(0);
///
/// assert_eq!(date, (1970, 1, 1));
/// ```

// TODO: Expand on docs a little bit?

// TODO (hard): potentially attempt to derive other calendars?

// TODO (easy): Add Julian calendar

pub mod gregorian;
