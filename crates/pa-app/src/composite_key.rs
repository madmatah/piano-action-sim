//! The composite key's mass, derived.
//!
//! The bench's standard key is a composite (`docs/FOUNDATIONS.md` §2.3,
//! `data/profiles/composite.json`): the owner's measured section, a capstan
//! and a key front no source gives. Until it is weighed, its mass and centre
//! of mass are derived from its volume: the keystick integrated as a uniform
//! prism of its profile's assumed width and density, and the steel capstan
//! turned from its profile ([`pa_core::parts::KeyDesign::mass_properties`]). Once it is weighed
//! ([`pa_core::parts::KeyMass`]), the derivation still gives the height of
//! its centre of mass, which no scale gives.
//!
//! The mechanism takes a key's mass as data, as it takes a weighed key's and
//! a published one's, and never computes it from an outline. This
//! adapter is where the composite key's is made, and the only place: it is
//! the composite key's derivation, and it leaves with the composite key.

use pa_core::mechanism::MassProps;
use pa_core::parts::KeyDesign;

/// The composite key's mass and centre of mass in its own frame, derived from
/// its volume; `None` for a design whose outline encloses nothing.
pub fn mass(key: &KeyDesign) -> Option<MassProps> {
    let derived = key.mass_properties()?;
    Some(MassProps {
        mass: derived.mass,
        com: derived.com,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The adapter hands on the volume model's numbers unchanged, bit for
    /// bit: the jig weighs the same composite key the closed forms on the
    /// frame weigh.
    #[test]
    fn the_derivation_is_the_volume_model() {
        let key = pa_core::test_fixture::key();
        let derived = key.mass_properties().unwrap();
        let adapted = mass(&key).unwrap();
        assert_eq!(adapted.mass.to_bits(), derived.mass.to_bits());
        assert_eq!(adapted.com, derived.com);
    }
}
