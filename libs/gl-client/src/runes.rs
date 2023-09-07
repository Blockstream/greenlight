use runeauth::{Alternative, Condition, Restriction, Rune, RuneError};
use std::fmt::Display;

/// Represents an entity that can provide restrictions.
///
/// The `Restrictor` trait should be implemented by types that are able to
/// produce a list of `Restriction`s. The `generate` method returns a `Result`
/// containing a vector of `Restriction`s or a `RuneError` in case of any error.
pub trait Restrictor {
    /// Retrieves the restrictions associated with the current instance.
    ///
    /// # Returns
    /// A `Result` containing a `Vec` of `Restriction`s. In the event of
    /// failure, returns a `RuneError`.
    fn generate(self) -> Result<Vec<Restriction>, RuneError>;
}

/// A factory responsible for carving runes.
///
/// `RuneFactory` provides utility functions to manipulate and produce runes
/// with certain characteristics, such as additional restrictions.
pub struct RuneFactory;

impl RuneFactory {
    /// Combines an original `Rune` with a list of restricters,
    /// and produces a new rune in base64 format.
    ///
    /// # Parameters
    /// - `origin`: A reference to the original `Rune` that will serve as the
    /// base.
    /// - `append`: A `Vec` containing entities that implement the `Restrictor`
    /// trait.
    ///
    /// # Returns
    /// A `Result` containing a `String` representing the carved rune in base64 format.
    /// In the event of any failure during the carving process, returns a `RuneError`.
    pub fn carve<T: Restrictor + Copy>(origin: &Rune, append: &[T]) -> Result<String, RuneError> {
        let restrictions = append.into_iter().try_fold(Vec::new(), |mut acc, res| {
            let mut r = res.generate()?;
            acc.append(&mut r);
            Ok(acc)
        })?;

        let mut originc = origin.clone();
        restrictions.into_iter().for_each(|r| {
            // Changes are applied in place, as well as returned, so
            // this is ok.
            let _ = originc.add_restriction(r);
        });

        Ok(originc.to_base64())
    }
}

/// Predefined rule sets to generate `Restriction`s from.
#[derive(Clone, Copy)]
pub enum DefRules<'a> {
    /// Represents a rule set where only read operations are allowed. This
    /// translates to a `Restriction` that is "method^Get|method^List".
    ReadOnly,
    /// Represents a rule set where only the `pay` method is allowed. This
    /// translates to a `Restriction` that is "method=pay".
    Pay,
    /// A special rule that adds the alternatives of the given `DefRules`
    /// in a disjunctive set. Example: Add(vec![ReadOnly, Pay]) translates
    /// to a `Restriction` that is "method^Get|method^List|method=pay".
    Add(&'a [DefRules<'a>]),
}

impl<'a> Restrictor for DefRules<'a> {
    /// Generate the actual `Restriction` entities based on the predefined rule
    /// sets.
    ///
    /// # Returns
    /// A `Result` containing a vector of `Restriction` entities or a `RuneError`
    /// if there's any error while generating the restrictions.
    fn generate(self) -> Result<Vec<Restriction>, RuneError> {
        match self {
            DefRules::ReadOnly => {
                let a: Vec<Restriction> = vec![Restriction::new(vec![
                    alternative("method", Condition::BeginsWith, "Get").unwrap(),
                    alternative("method", Condition::BeginsWith, "List").unwrap(),
                ])
                .unwrap()];
                Ok(a)
            }
            DefRules::Pay => {
                let a =
                    vec![Restriction::new(vec![
                        alternative("method", Condition::Equal, "pay").unwrap()
                    ])
                    .unwrap()];
                Ok(a)
            }
            DefRules::Add(rules) => {
                let alt_set =
                    rules
                        .into_iter()
                        .try_fold(Vec::new(), |mut acc: Vec<Alternative>, rule| {
                            let mut alts = rule
                                .generate()?
                                .into_iter()
                                .flat_map(|r| r.alternatives)
                                .collect();
                            acc.append(&mut alts);
                            Ok(acc)
                        })?;
                let a = vec![Restriction::new(alt_set)?];
                Ok(a)
            }
        }
    }
}

impl<'a> Display for DefRules<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefRules::ReadOnly => write!(f, "readonly"),
            DefRules::Pay => write!(f, "pay"),
            DefRules::Add(rules) => {
                write!(
                    f,
                    "{}",
                    rules.into_iter().fold(String::new(), |acc, r| {
                        if acc.is_empty() {
                            format!("{}", r)
                        } else {
                            format!("{}|{}", acc, r)
                        }
                    })
                )
            }
        }
    }
}

/// Creates an `Alternative` based on the provided field, condition, and value.
///
/// This function is a shorthand for creating new `Alternative` entities
/// without having to manually wrap field and value into `String`.
///
/// # Parameters
/// - `field`: The field on which the alternative is based.
/// - `cond`: The condition to check against the field.
/// - `value`: The value to match with the condition against the field.
///
/// # Returns
///
/// A result containing the created `Alternative` or a `RuneError` if there's
/// any error in the creation.
fn alternative(field: &str, cond: Condition, value: &str) -> Result<Alternative, RuneError> {
    Alternative::new(field.to_string(), cond, value.to_string(), false)
}

#[cfg(test)]
mod tests {
    use super::{DefRules, RuneFactory};
    use base64::{engine::general_purpose, Engine as _};
    use runeauth::Rune;

    #[test]
    fn test_carve_readonly_rune() {
        let seed = [0; 32];
        let mr = Rune::new_master_rune(&seed, vec![], None, None).unwrap();

        // Carve a new rune from the master rune with given restrictions.
        let carved = RuneFactory::carve(&mr, &[DefRules::ReadOnly]).unwrap();

        let carved_byt = general_purpose::URL_SAFE.decode(&carved).unwrap();
        let carved_restr = String::from_utf8(carved_byt[32..].to_vec()).unwrap(); // Strip off the authcode to inspect the restrictions.
        assert_eq!(carved_restr, *"method^Get|method^List");

        let carved_rune = Rune::from_base64(&carved).unwrap();
        assert!(mr.is_authorized(&carved_rune));
    }

    #[test]
    fn test_carve_disjunction_rune() {
        let seed = [0; 32];
        let mr = Rune::new_master_rune(&seed, vec![], None, None).unwrap();

        // Carve a new rune from the master rune with given restrictions.
        let carved =
            RuneFactory::carve(&mr, &[DefRules::Add(&[DefRules::ReadOnly, DefRules::Pay])])
                .unwrap();

        let carved_byt = general_purpose::URL_SAFE.decode(&carved).unwrap();
        let carved_restr = String::from_utf8(carved_byt[32..].to_vec()).unwrap(); // Strip off the authcode to inspect the restrictions.
        assert_eq!(carved_restr, *"method^Get|method^List|method=pay");

        let carved_rune = Rune::from_base64(&carved).unwrap();
        assert!(mr.is_authorized(&carved_rune));
    }

    #[test]
    fn test_defrules_display() {
        let r = DefRules::Pay;
        assert_eq!(format!("{}", r), "pay");
        let r = DefRules::Add(&[DefRules::Pay]);
        assert_eq!(format!("{}", r), "pay");
        let r = DefRules::Add(&[DefRules::Pay, DefRules::ReadOnly]);
        assert_eq!(format!("{}", r), "pay|readonly");
    }
}
